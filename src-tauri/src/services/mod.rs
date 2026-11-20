pub mod access;
pub mod audit;
pub mod backup_archive;
pub mod backups;
pub mod company;
pub mod crypto;
pub mod key_store;
pub mod orders;
pub mod portal;
pub mod portal_publisher;
pub mod portal_transport;
pub mod recovery;
pub mod users;

use crate::{
    db::Database,
    error::AppError,
    repositories::{access::AccessRepository, audit::AuditRepository, orders::OrderRepository},
};
use std::sync::Arc;

pub struct BackendServices {
    pub portal: Arc<portal::PortalService>,
    pub backups: backups::BackupService,
    pub evidence: evidence::EvidenceService,
    pub reports: reports::ReportService,
    pub repairs: repairs::RepairService,
    pub ledger: ledger::LedgerService,
    pub access: Arc<access::AccessService>,
    pub orders: orders::OrderService,
    pub company: company::CompanyService,
    pub users: users::UserService,
}
impl BackendServices {
    pub async fn with_os_key_store(
        database: Database,
        key_directory: &std::path::Path,
    ) -> Result<Self, AppError> {
        let mut connection = database.pool().acquire().await?;
        let needs_existing_key = OrderRepository.has_credentials(&mut connection).await?;
        drop(connection);
        let keys = if needs_existing_key {
            key_store::OsKeyStore::open_existing(key_directory)?
        } else {
            key_store::OsKeyStore::open(key_directory)?
        };
        Self::new(database, Arc::new(keys)).await
    }

    pub async fn new(
        database: Database,
        keys: Arc<dyn crypto::KeyProvider>,
    ) -> Result<Self, AppError> {
        let pool = database.pool().clone();
        let assets = database.asset_directory();
        let audit = Arc::new(audit::AuditService::new(Arc::new(AuditRepository)));
        let access = Arc::new(
            access::AccessService::new(pool.clone(), Arc::new(AccessRepository), audit.clone())
                .await?,
        );
        access.recover().await?;
        let root = assets
            .as_ref()
            .and_then(|path| path.parent())
            .map(|path| path.to_path_buf());
        let backups = backups::BackupService::new(
            pool.clone(),
            Arc::new(crate::repositories::backups::BackupRepository),
            access.clone(),
            keys.clone(),
            root.clone(),
        );
        let crypto = Arc::new(crypto::CryptoService::new(keys));
        let order_repository = Arc::new(OrderRepository);
        let portal_repository = Arc::new(crate::repositories::portal::PortalRepository);
        let publication = Arc::new(portal::PublicationQueue::new(
            portal_repository.clone(),
            audit.clone(),
        ));
        let portal = Arc::new(portal::PortalService::new(
            pool.clone(),
            portal_repository,
            publication.clone(),
            access.clone(),
            audit.clone(),
            root,
        )?);
        let ledger_repository = Arc::new(crate::repositories::ledger::LedgerRepository);
        let users = users::UserService::new(
            pool.clone(),
            Arc::new(crate::repositories::users::UserRepository),
            access.clone(),
            audit.clone(),
        );
        let company = company::CompanyService::new(
            pool.clone(),
            Arc::new(crate::repositories::company::CompanyRepository),
            access.clone(),
            audit.clone(),
            assets.clone(),
        );
        let repairs = repairs::RepairService::new(
            pool.clone(),
            Arc::new(crate::repositories::repairs::RepairRepository),
            ledger_repository.clone(),
            access.clone(),
            audit.clone(),
            publication.clone(),
        );
        let ledger = ledger::LedgerService::new(
            pool.clone(),
            ledger_repository.clone(),
            access.clone(),
            audit.clone(),
            publication.clone(),
        );
        let reports = reports::ReportService::new(
            pool.clone(),
            Arc::new(crate::repositories::reports::ReportRepository),
            order_repository.clone(),
            access.clone(),
            audit.clone(),
        );
        let evidence = evidence::EvidenceService::new(
            pool.clone(),
            Arc::new(crate::repositories::evidence::EvidenceRepository),
            order_repository.clone(),
            ledger_repository,
            access.clone(),
            audit.clone(),
            assets,
        );
        let orders = orders::OrderService::new(
            pool,
            order_repository,
            access.clone(),
            audit,
            crypto,
            publication,
        );
        Ok(Self {
            portal,
            backups,
            access,
            orders,
            company,
            users,
            repairs,
            ledger,
            reports,
            evidence,
        })
    }
}

#[cfg(test)]
mod tests;

pub mod ledger;
pub mod repairs;

pub mod reports;

pub mod evidence;
