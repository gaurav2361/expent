use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Transactions: user_id + date (for listing transactions)
        manager
            .create_index(
                Index::create()
                    .name("idx-transactions-user_id-date")
                    .table(Transactions::Table)
                    .col(Transactions::UserId)
                    .col(Transactions::Date)
                    .to_owned(),
            )
            .await?;

        // Transactions: group_id
        manager
            .create_index(
                Index::create()
                    .name("idx-transactions-group_id")
                    .table(Transactions::Table)
                    .col(Transactions::GroupId)
                    .to_owned(),
            )
            .await?;

        // Subscriptions: user_id
        manager
            .create_index(
                Index::create()
                    .name("idx-subscriptions-user_id")
                    .table(Subscriptions::Table)
                    .col(Subscriptions::UserId)
                    .to_owned(),
            )
            .await?;

        // P2P Requests: receiver_email
        manager
            .create_index(
                Index::create()
                    .name("idx-p2p_requests-receiver_email")
                    .table(P2PRequests::Table)
                    .col(P2PRequests::ReceiverEmail)
                    .to_owned(),
            )
            .await?;

        // Contact Identifiers: value
        manager
            .create_index(
                Index::create()
                    .name("idx-contact_identifiers-value")
                    .table(ContactIdentifiers::Table)
                    .col(ContactIdentifiers::Value)
                    .to_owned(),
            )
            .await?;

        // Contact Identifiers: contact_id
        manager
            .create_index(
                Index::create()
                    .name("idx-contact_identifiers-contact_id")
                    .table(ContactIdentifiers::Table)
                    .col(ContactIdentifiers::ContactId)
                    .to_owned(),
            )
            .await?;

        // Txn Parties: transaction_id
        manager
            .create_index(
                Index::create()
                    .name("idx-txn_parties-transaction_id")
                    .table(TxnParties::Table)
                    .col(TxnParties::TransactionId)
                    .to_owned(),
            )
            .await?;

        // User UPI IDs: user_id
        manager
            .create_index(
                Index::create()
                    .name("idx-user_upi_ids-user_id")
                    .table(UserUpiIds::Table)
                    .col(UserUpiIds::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-transactions-user_id-date")
                    .table(Transactions::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-transactions-group_id")
                    .table(Transactions::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-subscriptions-user_id")
                    .table(Subscriptions::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-p2p_requests-receiver_email")
                    .table(P2PRequests::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-contact_identifiers-value")
                    .table(ContactIdentifiers::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-contact_identifiers-contact_id")
                    .table(ContactIdentifiers::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-txn_parties-transaction_id")
                    .table(TxnParties::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-user_upi_ids-user_id")
                    .table(UserUpiIds::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Transactions {
    Table,
    UserId,
    Date,
    GroupId,
}

#[derive(DeriveIden)]
enum Subscriptions {
    Table,
    UserId,
}

#[derive(DeriveIden)]
enum P2PRequests {
    Table,
    ReceiverEmail,
}

#[derive(DeriveIden)]
enum ContactIdentifiers {
    Table,
    Value,
    ContactId,
}

#[derive(DeriveIden)]
enum TxnParties {
    Table,
    TransactionId,
}

#[derive(DeriveIden)]
enum UserUpiIds {
    Table,
    UserId,
}
