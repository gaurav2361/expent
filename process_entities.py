import os
import re

# This script post-processes SeaORM entities to add ts-rs attributes for TypeScript type generation.
# It ensures that:
# 1. Models are renamed from 'Model' to their singular entity name (e.g. Account, User).
# 2. Types are exported directly to packages/types/src/db/.
# 3. Decimal and Json fields are correctly annotated for TypeScript.
# 4. No duplicate attributes are created.

files_to_process = [
    "accounts.rs",
    "bank_statement_rows.rs",
    "budgets.rs",
    "categories.rs",
    "contact_identifiers.rs",
    "contact_links.rs",
    "contact_staging.rs",
    "contacts.rs",
    "groups.rs",
    "ledger_tabs.rs",
    "ocr_job_edits.rs",
    "ocr_jobs.rs",
    "p2p_requests.rs",
    "p2p_transfers.rs",
    "purchase_imports.rs",
    "purchase_items.rs",
    "purchases.rs",
    "sessions.rs",
    "statement_txn_matches.rs",
    "sub_alerts.rs",
    "subscription_charges.rs",
    "subscriptions.rs",
    "transaction_edits.rs",
    "transaction_metadata.rs",
    "transaction_sources.rs",
    "transactions.rs",
    "txn_parties.rs",
    "user_groups.rs",
    "user_upi_ids.rs",
    "users.rs",
    "verifications.rs",
    "wallets.rs",
]

base_dir = "crates/db/src/entities"


def to_singular_pascal_case(snake_str):
    name = snake_str.replace(".rs", "")
    if name.endswith("ies"):
        name = name[:-3] + "y"
    elif name.endswith("ches") or name.endswith("shes") or name.endswith("sses"):
        name = name[:-2]
    elif name.endswith("s") and not name.endswith("ss"):
        name = name[:-1]

    if name == "contact_staging":
        name = "contact_staging"

    return "".join(x.capitalize() for x in name.split("_"))


for filename in files_to_process:
    filepath = os.path.join(base_dir, filename)
    if not os.path.exists(filepath):
        continue

    with open(filepath, "r") as f:
        content = f.read()

    # 1. Add ts_rs import
    if "use ts_rs::TS;" not in content:
        content = content.replace(
            "use sea_orm::entity::prelude::*;", "use sea_orm::entity::prelude::*;\nuse ts_rs::TS;"
        )

    entity_name = to_singular_pascal_case(filename)

    # 2. Add TS to derive block
    # Matches #[derive(... DeriveModel ...)] and adds TS if missing
    derive_pattern = r"#\[derive\(([^)]*DeriveModel[^)]*)\)\]"

    def add_ts_to_derive(match):
        traits = [t.strip() for t in match.group(1).split(",") if t.strip()]
        if "TS" not in traits:
            traits.append("TS")
        return f"#[derive({', '.join(traits)})]"

    content = re.sub(derive_pattern, add_ts_to_derive, content)

    # 3. Handle #[ts(export...)] attribute (Clean & Re-add)
    # Remove any existing multi-line or single-line #[ts(export...)] blocks
    content = re.sub(r'#\[ts\(\s*export,.*?export_to = "[^"]*"\s*\)\]\n', "", content, flags=re.DOTALL)

    ts_export_new = (
        f'#[ts(export, rename = "{entity_name}", export_to = "../../../packages/types/src/db/{entity_name}.ts")]\n'
    )
    # Insert right before pub struct Model
    content = content.replace("pub struct Model {", f"{ts_export_new}pub struct Model {{")

    # 4. Field Type Fixes (Id and PrimaryKeyTrait)
    content = content.replace("pub id: Option<String>,", "pub id: String,")
    content = content.replace("type ValueType = Option<String>;", "type ValueType = String;")

    # 5. Decimal Annotations (Clean & Re-add)
    # Remove existing decimal attributes
    content = re.sub(r'#\[ts\(type = "string( \| null)?"\)\]\n\s+', "", content)
    # Add them back
    content = re.sub(r"(pub [a-z0-9_]+: Decimal,)", r'#[ts(type = "string")]\n    \1', content)
    content = re.sub(r"(pub [a-z0-9_]+: Option<Decimal>,)", r'#[ts(type = "string | null")]\n    \1', content)

    # 6. Json Annotations (Clean & Re-add)
    # Remove existing json attributes
    content = re.sub(r'#\[ts\(as = "(Option<)?crate::ExportedJsonValue(>)?"\)\]\n\s+', "", content)
    # Add them back
    content = re.sub(r"(pub [a-z0-9_]+: Json,)", r'#[ts(as = "crate::ExportedJsonValue")]\n    \1', content)
    content = re.sub(
        r"(pub [a-z0-9_]+: Option<Json>,)", r'#[ts(as = "Option<crate::ExportedJsonValue>")]\n    \1', content
    )

    with open(filepath, "w") as f:
        f.write(content)

print("Entities post-processed successfully.")
