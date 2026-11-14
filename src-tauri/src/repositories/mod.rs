pub mod access;
pub mod audit;
pub mod company;
pub mod orders;
pub mod users;

pub mod ledger;
pub mod repairs;

pub mod reports;

pub mod evidence;

pub type RepositoryFuture<'a, T> = std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<T, crate::error::AppError>> + Send + 'a>,
>;
