use std::fmt::Display;

use axum::http::StatusCode;

use crate::base::{
    query::{
        filter::{Filter, FilterChain, FilterCondition, FilterOperator},
        BasableQuery, QueryOperation, QueryOrder,
    },
    AppError,
};

#[derive(Clone)]
pub enum TrendAnalysisType {
    IntraModel,
    CrossModel,
}

pub enum TrendAnalysisOrder {
    DESC,
    ASC,
}

impl Display for TrendAnalysisOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let order = match self {
            TrendAnalysisOrder::DESC => "DESC",
            TrendAnalysisOrder::ASC => "ASC",
        };

        write!(f, "{}", order)
    }
}

pub struct CrossOptions {
    /// The foreign table we want to connect to.
    pub foreign_table: String,

    /// Column on [`TrendAnalysisOpts::table`] that should be targeted by [`TrendAnalysisOpts::ycol`].
    pub target_col: String,
}

pub struct TrendAnalysisOpts {
    /// The primary table you want to analyze
    pub table: String,

    /// The type of trend analysis you want want to perform
    pub analysis_type: TrendAnalysisType,

    /// The column you want to use for independent variables.
    pub xcol: String,

    /// The column you want to use for dependent variable. For [`TrendAnalysisType::CrossModel`],
    /// this should be set to the column name used as foreignKey from [`CrossOptions::foreign_table`].
    pub ycol: String,

    /// Order of the analysis
    pub order: TrendAnalysisOrder,

    /// Limit of returned analysis
    pub limit: usize,

    /// Configure this option if you're using [`TrendAnalysisType::CrossModel`].
    pub cross: Option<CrossOptions>,
}

impl TryFrom<TrendAnalysisOpts> for BasableQuery {
    type Error = AppError;

    fn try_from(value: TrendAnalysisOpts) -> Result<Self, Self::Error> {
        let TrendAnalysisOpts {
            table,
            analysis_type,
            xcol,
            ycol,
            order,
            limit,
            cross,
        } = value;

        match analysis_type {
            TrendAnalysisType::IntraModel => {
                let operation = QueryOperation::SelectData(Some(vec![xcol, ycol.clone()]));
                
                let order = match order {
                    TrendAnalysisOrder::DESC => QueryOrder::DESC(ycol),
                    TrendAnalysisOrder::ASC => QueryOrder::ASC(ycol),
                };

                let q = BasableQuery {
                    table,
                    operation,
                    order_by: Some(order),
                    limit: Some(limit),
                    ..Default::default()
                };

                Ok(q)
            }

            TrendAnalysisType::CrossModel => match cross {
                Some(cross) => {
                    let CrossOptions {
                        foreign_table,
                        target_col,
                    } = cross;

                    let select_columns = vec![
                        format!("x.{xcol} AS {xcol}"),
                        format!("COUNT(y.{ycol}) AS {ycol}"),
                    ];

                    let operation = QueryOperation::SelectData(Some(select_columns));
                    let left_join = format!("{foreign_table} y ON x.{target_col} = y.{ycol}");
                    
                    let mut having = FilterChain::new();
                    having.add_one(Filter::BASE(
                        FilterCondition {
                        column: ycol.clone(),
                        operator: FilterOperator::Gt("0".to_string()),
                    }));

                    let order = match order {
                        TrendAnalysisOrder::DESC => QueryOrder::DESC(ycol),
                        TrendAnalysisOrder::ASC => QueryOrder::ASC(ycol),
                    };

                    let q = BasableQuery {
                        operation,
                        having,
                        table: format!("{table} x"),
                        left_join: Some(left_join),
                        group_by: Some(vec![xcol]),
                        order_by: Some(order),
                        limit: Some(limit),
                        ..Default::default()
                    };

                    Ok(q)
                }
                None => {
                    let err = AppError::new(
                        StatusCode::EXPECTATION_FAILED,
                        "You must provide cross model options.",
                    );
                    
                    Err(err)
                }
            },
        }
    }
}
