use google_cloud_gax::cancel::CancellationToken;
use google_cloud_googleapis::spanner::v1::execute_sql_request::QueryMode;
use google_cloud_spanner::client::ReadWriteTransactionOption;
use google_cloud_spanner::transaction::{Transaction, CallOptions, QueryOptions};
use google_cloud_spanner::transaction_rw::CommitOptions;

#[derive(Clone)]
pub struct Context {
    pub cancel: CancellationToken,
}

impl Context {
    pub fn new(cancel: CancellationToken) -> Self {
        Self {
            cancel,
        }
    }
}

impl From<&mut Context> for QueryOptions {
    fn from(ctx: &mut Context) -> Self {
        Self {
            mode: QueryMode::Normal,
            optimizer_options: None,
            call_options: CallOptions {
                cancel: Some(ctx.cancel.clone()),
                ..Default::default()
            },
        }
    }
}

impl From<&mut Context> for CallOptions {
    fn from(ctx: &mut Context) -> Self {
        Self {
            cancel: Some(ctx.cancel.clone()),
            ..Default::default()
        }
    }
}

impl From<&mut Context> for ReadWriteTransactionOption {
    fn from(ctx: &mut Context) -> Self {
        Self {
            begin_options: (&mut ctx.clone()).into(),
            commit_options: CommitOptions {
                return_commit_stats: false,
                call_options: ctx.into(),
            }
        }
    }
}