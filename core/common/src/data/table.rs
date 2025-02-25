use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    query::{
        filter::{Filter, FilterChain},
        BasableQuery, QueryCommand, QueryOrder,
    },
    error::AppError,
};

pub type TableSummaries = Vec<TableSummary>;

pub type DataQueryResult<V, E> = Result<Vec<HashMap<String, V>>, E>;

/// Table column used for querying table history such as when a row was added or when a row was updated.
#[derive(Deserialize, Serialize, Clone)]
pub struct HistoryColumn {
    name: String,
    pattern: String,
}

/// The type of `SpecialColumn`
#[derive(Deserialize, Serialize, Clone)]
pub enum SpecialValueType {
    Image,
    Audio,
    Video,
    PDF,
    Webpage,
}

/// Special columns are columns whose values should lead to some sort of media types.
#[derive(Deserialize, Serialize, Clone)]
pub struct SpecialColumn {
    name: String,
    special_type: SpecialValueType,
    path: String,
}

/// The action that should trigger `NotifyEvent`.
#[derive(Deserialize, Serialize, Clone)]
enum NotifyTrigger {
    Create,
    Update,
    Delete,
}

/// When should `NotifyEvent` get triggered around `NotifyTrigger`.
#[derive(Deserialize, Serialize, Clone)]
pub enum NotifyTriggerTime {
    Before,
    After,
}

/// The REST API method expected by the webhook URL.
#[derive(Deserialize, Serialize, Clone)]
pub enum NotifyEventMethod {
    Get,
    Post,
    Delete,
    Put,
    Patch,
}

/// What should happen to the operation `NotifyTrigger` when there's notification error?
/// Let's say there's a server error from the webhook URL, should we proceed or fail the operation?
#[derive(Deserialize, Serialize, Clone)]
pub enum OnNotifyError {
    Fail,
    Proceed,
}

/// Event sent to a given webhook URL based on certain `NotifyTrigger`
#[derive(Deserialize, Serialize, Clone)]
pub struct NotifyEvent {
    trigger: NotifyTrigger,
    trigger_time: NotifyTriggerTime,
    method: NotifyEventMethod,
    url: String,
    on_error: OnNotifyError,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TableConfig {
    pub label: String,

    pub name: String,

    /// Name of column to use as primary key.
    pub pk_column: Option<String>,

    /// Total number of items to be loaded for each pagination
    pub items_per_page: usize,

    /// Column for querying when a row was created.
    pub created_column: Option<HistoryColumn>,

    /// Column for querying when a row was updated.
    pub updated_column: Option<HistoryColumn>,

    /// Special columns that return `SpecialValueType`
    pub special_columns: Option<Vec<SpecialColumn>>,

    /// Notification events for this table.
    pub events: Option<Vec<NotifyEvent>>,

    /// Columns to exclude from fetch query
    pub exclude_columns: Option<Vec<String>>,
}

impl PartialEq for TableConfig {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl Default for TableConfig {
    fn default() -> Self {
        TableConfig {
            pk_column: None,
            label: String::new(),
            name: String::new(),
            items_per_page: 100,
            created_column: None,
            updated_column: None,
            special_columns: None,
            events: None,
            exclude_columns: None,
        }
    }
}

#[derive(Deserialize)]
pub struct TableQueryOpts {
    /// The table we're querying
    pub table: String,

    /// Query offset
    pub offset: usize,

    /// Query row count
    pub row_count: usize,

    /// Query filters
    pub filters: Option<Vec<Filter>>,

    /// The columns(s) you want selected in the query. If set to `None` all fields
    /// will be selected.
    pub columns: Option<Vec<String>>,

    pub order_by: Option<QueryOrder>,
    pub search_opts: Option<TableSearchOpts>,
}

impl TableQueryOpts {
    pub fn is_search_mode(&self) -> bool {
        self.search_opts.is_some()
    }
}

impl TryFrom<TableQueryOpts> for BasableQuery {
    type Error = AppError;

    fn try_from(opts: TableQueryOpts) -> Result<Self, Self::Error> {
        let TableQueryOpts {
            table,
            offset,
            row_count,
            filters,
            columns,
            order_by,
            search_opts,
        } = opts;

        let operation = QueryCommand::SelectData(columns);
        let filter_chain = filters.map_or(FilterChain::empty(), |filters| {
            FilterChain::prefill(filters)
        });

        let bq = BasableQuery {
            table,
            command: operation,
            row_count: Some(row_count),
            offset: Some(offset),
            filters: filter_chain,
            order_by,
            search_opts,
            ..Default::default()
        };

        Ok(bq)
    }
}

#[derive(Deserialize)]
pub struct TableSearchOpts {
    pub search_cols: Vec<String>,
    pub query: String,
}

#[derive(Serialize)]
pub struct TableSummary {
    pub name: String,
    pub row_count: u32,
    pub col_count: u32,
    pub created: Option<String>,
    pub updated: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct UpdateTableData {
    pub unique_key: String,
    pub columns: Vec<String>,
    pub unique_values: Vec<String>,
    pub input: Vec<HashMap<String, String>>,
}

#[derive(Deserialize, Clone)]
pub enum TableExportFormat {
    CSV,
    TSV,
    PSV,
    TEXT,
    JSON,
    HTML,
    XML,
}

impl TableExportFormat {
    pub fn as_extension(&self) -> String {
        let ext = match self {
            TableExportFormat::CSV => "csv",
            TableExportFormat::TSV => "tsv",
            TableExportFormat::PSV => "psv",
            TableExportFormat::TEXT => "txt",
            TableExportFormat::JSON => "json",
            TableExportFormat::HTML => "html",
            TableExportFormat::XML => "xml",
        };

        ext.to_string()
    }

    pub fn as_mimetype(&self) -> String {
        let mime_type = match self {
            TableExportFormat::CSV => "text/csv",
            TableExportFormat::TSV => "text/tab-separated-values",
            TableExportFormat::PSV => "text/plain", // No specific MIME type; plain text is the closest match
            TableExportFormat::TEXT => "text/plain",
            TableExportFormat::JSON => "application/json",
            TableExportFormat::HTML => "text/html",
            TableExportFormat::XML => "application/xml", // Or "text/xml" based on context
        };
        
        mime_type.to_string()
    }

    pub fn field_delimiter(&self) -> Option<String> {
        let dlm = match self {
            TableExportFormat::CSV => Some(","),
            TableExportFormat::TSV => Some("\\t"),
            TableExportFormat::PSV => Some("|"),
            TableExportFormat::TEXT => Some(";"),
            _ => None
        };

        dlm.map(|dlm_str| dlm_str.to_string())
    }
}

#[derive(Deserialize)]
pub struct TableExportTrim {
    pub offset: usize,
    pub count: usize
}

#[derive(Deserialize)]
pub struct TableExportOpts {
    pub format: TableExportFormat,
    pub query_opts: TableQueryOpts,
    pub trim: Option<TableExportTrim>
}

#[derive(Serialize)]
pub struct  TableExportResponse {
    pub data: String,
    pub mimetype: String,
    pub filename: String
}
