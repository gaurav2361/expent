pub mod create;
pub mod delete;
pub mod get;
pub mod identifiers;
pub mod list;
pub mod update;

pub use create::create_contact;
pub use delete::delete_contact;
pub use get::get_contact_detail;
pub use identifiers::add_contact_identifier;
pub use list::list_contacts;
pub use update::update_contact;
