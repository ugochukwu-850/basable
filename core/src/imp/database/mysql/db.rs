use std::{collections::HashMap, sync::Arc};

use mysql::{DriverError::SetupError, Row, Value};
use time::Date;
use uuid::Uuid;

use crate::{
    base::{
        config::ConnectionConfig,
        data::table::{TableSummaries, TableSummary},
        imp::{
            analysis::{
                category::CategoryGraphOpts,
                chrono::{ChronoAnalysisBasis, ChronoAnalysisOpts},
                trend::{TrendAnalysisOpts, TrendAnalysisType},
                AnalysisResult, AnalysisResults, AnalysisValue, VisualizeDB,
            },
            db::{DBError, DB},
            table::Table,
            ConnectorType, SharedTable,
        },
        AppError,
    },
    imp::database::{DBVersion, DbConnectionDetails},
};

use super::{table::MySqlTable, MySqlValue};

pub(crate) struct MySqlDB {
    pub connector: ConnectorType,
    pub tables: Vec<SharedTable>,
    user_id: String,
    id: Uuid,
}

impl MySqlDB {
    pub fn new(connector: ConnectorType, user_id: String) -> Self {
        MySqlDB {
            connector,
            tables: Vec::new(),
            user_id,
            id: Uuid::new_v4(),
        }
    }

    /// Get MySQL server version and host OS version
    fn show_version(&self) -> Result<DBVersion, AppError> {
        let vars = self.exec_query(
            "
                SHOW VARIABLES 
                WHERE Variable_name 
                IN (
                    'version_compile_os', 
                    'version', 
                    'version_comment', 
                    'version_compile_zlib'
                )
            ",
        )?;

        let mut data = HashMap::new();

        for v in vars {
            let name: String = v.get("Variable_name").unwrap();
            let value: String = v.get("Value").unwrap();
            data.insert(name, value);
        }

        Ok(data)
    }

    fn size(&self) -> Result<f64, AppError> {
        let db = self.config().db_name.as_ref().unwrap();

        let query = format!(
            "
            SELECT table_schema '{db}', 
            ROUND(SUM(data_length + index_length) / 1024 / 1024, 1) 'size' 
            FROM information_schema.tables 
            WHERE table_schema = '{db}'
            GROUP BY table_schema
        "
        );

        let qr = self.exec_query(&query)?;

        // db size is returned in MB, we may want to write a function
        // to convert for GB, TB...etc
        let size: f64 = qr.first().map_or(0.0, |r| {
            let s: String = r.get("size").unwrap();
            s.parse().unwrap()
        });

        Ok(size)
    }

    fn config(&self) -> &ConnectionConfig {
        &self.connector.config()
    }

    fn exec_query(&self, query: &str) -> mysql::Result<Vec<Row>> {
        self.connector.exec_query(query)
    }
}

impl DB for MySqlDB {
    type Error = mysql::Error;
    type Row = mysql::Row;
    type ColumnValue = MySqlValue;

    fn id(&self) -> &Uuid {
        &self.id
    }

    fn user_id(&self) -> &str {
        &self.user_id
    }

    fn connector(&self) -> &ConnectorType {
        &self.connector
    }

    fn load_tables(&mut self, connector: ConnectorType) -> Result<(), AppError> {
        let tables = self.query_tables()?;

        if !tables.is_empty() {
            tables.iter().for_each(|t| {
                let connector = connector.clone();
                let name: String = t.get("TABLE_NAME").unwrap();

                let table = MySqlTable::new(name, connector);
                self.tables.push(Arc::new(table));
            })
        }

        Ok(())
    }

    fn tables(&self) -> &Vec<SharedTable> {
        &self.tables
    }

    fn query_tables(&self) -> mysql::Result<Vec<Row>> {
        let query = format!(
            "
                SELECT table_name, table_rows, create_time, update_time
                FROM information_schema.tables
                WHERE table_schema = '{}'
                ORDER BY table_name;
            ",
            self.config().db_name.clone().unwrap()
        );

        self.connector.exec_query(&query)
    }

    fn query_table_summaries(&self) -> Result<TableSummaries, AppError> {
        let results = self.query_tables()?;
        let tables: Vec<TableSummary> = results
            .iter()
            .map(|res| {
                let created = res.get("CREATE_TIME") as Option<Date>;
                let updated = res.get("CREATE_TIME") as Option<Date>;
                let name: String = res.get("TABLE_NAME").unwrap();

                let col_count = self.query_column_count(&name).unwrap();

                TableSummary {
                    name,
                    col_count,
                    row_count: res.get("TABLE_ROWS").unwrap(),
                    created: created.map_or(None, |d| Some(d.to_string())),
                    updated: updated.map_or(None, |d| Some(d.to_string())),
                }
            })
            .collect();

        Ok(tables)
    }

    fn query_column_count(&self, tb_name: &str) -> Result<u32, AppError> {
        let query = format!(
            "
                SELECT count(*) 
                FROM information_schema.columns 
                WHERE table_schema = '{}' and table_name = '{}'
                ORDER BY table_name;
            ",
            self.config().db_name.clone().unwrap(),
            tb_name
        );

        let qr = self.exec_query(&query)?;
        let c: u32 = qr.first().map_or(0, |r| r.get("count(*)").unwrap());

        Ok(c)
    }

    fn get_table(&self, name: &str) -> Option<&SharedTable> {
        self.tables.iter().find(|t| t.name() == name)
    }

    fn details(&self) -> Result<DbConnectionDetails, AppError> {
        let version = self.show_version()?;
        let tables = self.query_table_summaries()?;
        let size = self.size()?;
        let id = self.id.clone();
        let id = id.to_string();

        Ok(DbConnectionDetails {
            id,
            tables,
            version,
            db_size: size,
        })
    }
}

impl VisualizeDB for MySqlDB {
    fn chrono_graph(&self, opts: ChronoAnalysisOpts) -> Result<AnalysisResults, DBError> {
        let ChronoAnalysisOpts {
            table,
            chrono_col,
            basis,
            range,
        } = opts;

        let start = range.start();
        let end = range.end();

        let xcol = "BASABLE_CHRONO_BASIS_VALUE";
        let ycol = "BASABLE_CHRONO_RESULT";

        let query = format!(
            "
            SELECT
                {basis}({chrono_col}) as {xcol},
                COUNT(*) as {ycol}
            FROM
                {table}
            WHERE
                {chrono_col} BETWEEN '{start}' AND '{end}'
            GROUP BY
                {basis}({chrono_col})
            ORDER BY
                BASABLE_CHRONO_BASIS_VALUE

        "
        );

        let conn = self.connector();
        let rows = conn.exec_query(&query)?;

        let results: AnalysisResults = rows
            .iter()
            .map(|r| {
                let x = match basis {
                    ChronoAnalysisBasis::Daily => {
                        let date: Date = r.get(xcol).unwrap();
                        AnalysisValue::Date(date)
                    }
                    _ => AnalysisValue::UInt(r.get(xcol).unwrap()),
                };

                let y = AnalysisValue::UInt(r.get(ycol).unwrap());

                AnalysisResult::new(x, y)
            })
            .collect();

        Ok(results)
    }

    fn trend_graph(&self, opts: TrendAnalysisOpts) -> Result<AnalysisResults, DBError> {
        let query = opts
            .build_query()
            .map_err(|_| mysql::Error::DriverError(SetupError));
        let query = query?;

        let TrendAnalysisOpts {
            xcol,
            ycol,
            analysis_type,
            ..
        } = opts;

        let conn = self.connector();
        let rows = conn.exec_query(&query)?;

        let results: AnalysisResults = rows
            .iter()
            .map(|r| {
                let x = AnalysisValue::Text(r.get(xcol.as_str()).unwrap());
                let y = match analysis_type {
                    TrendAnalysisType::IntraModel => {
                        AnalysisValue::Double(r.get(ycol.as_str()).unwrap())
                    }
                    TrendAnalysisType::CrossModel => {
                        AnalysisValue::UInt(r.get(ycol.as_str()).unwrap())
                    }
                };

                AnalysisResult::new(x, y)
            })
            .collect();

        Ok(results)
    }

    fn category_graph(&self, opts: CategoryGraphOpts) -> Result<AnalysisResults, AppError> {
        let CategoryGraphOpts {
            table,
            graph_type,
            target_col,
            limit,
        } = opts;
        let query = format!(
            "
                SELECT COUNT(*) as COUNT, {target_col}
                FROM {table}
                GROUP BY {target_col}
                LIMIT {limit}
            "
        );

        let conn = self.connector();

        let rows = conn.exec_query(&query)?;
        let results: AnalysisResults = rows.iter().map(|r| {
            let x = AnalysisValue::UInt(r.get("COUNT").unwrap());

            let y_value: Value = r.get(target_col.as_str()).unwrap();
            let y = y_value.into();

            AnalysisResult::new(x, y)
        }).collect();

        Ok(results)
    }
}
