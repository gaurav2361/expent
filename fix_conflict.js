const fs = require('fs');
let file = "crates/expent_core/src/services/ocr/process.rs";
let content = fs.readFileSync(file, 'utf8');

const search = `<<<<<<< HEAD
            let mut contact_id = processed
                .data
                .0
                .get("contact_id")
                .and_then(|v| v.as_str())
                .map(std::string::ToString::to_string);

            let wallet_id = processed
                .data
                .0
                .get("wallet_id")
                .and_then(|v| v.as_str())
                .map(std::string::ToString::to_string);
=======
>>>>>>> origin/main`;

content = content.replace(search, "");
fs.writeFileSync(file, content);
console.log("Conflict resolved.");
