//! In-memory demo database: connections and deterministic row generation.

use tablepro_domain::{
    ColType, ConnectOutcome, Connection, Engine, Environment, SafeMode, Table, Value,
};

pub fn connections() -> Vec<Connection> {
    vec![
        Connection {
            name: "Local PostgreSQL".into(),
            engine: Engine::Postgres,
            host: "localhost".into(),
            port: 5432,
            database: "acme_dev".into(),
            user: "postgres".into(),
            environment: Environment::Local,
            safe_mode: SafeMode::Silent,
            ssl: false,
            ssh: None,
            group: "Personal".into(),
            last_used: "2 minutes ago".into(),
            outcome: ConnectOutcome::Ok,
        },
        Connection {
            name: "Development".into(),
            engine: Engine::Postgres,
            host: "dev-db.internal.acme.io".into(),
            port: 5432,
            database: "acme_dev".into(),
            user: "acme_app".into(),
            environment: Environment::Development,
            safe_mode: SafeMode::Silent,
            ssl: true,
            ssh: None,
            group: "Acme".into(),
            last_used: "yesterday".into(),
            outcome: ConnectOutcome::Ok,
        },
        Connection {
            name: "Staging".into(),
            engine: Engine::Postgres,
            host: "staging-db.acme.io".into(),
            port: 5432,
            database: "acme_staging".into(),
            user: "acme_app".into(),
            environment: Environment::Staging,
            safe_mode: SafeMode::Alert,
            ssl: true,
            ssh: Some("bastion.acme.io".into()),
            group: "Acme".into(),
            last_used: "3 days ago".into(),
            outcome: ConnectOutcome::AuthFailed,
        },
        Connection {
            name: "Analytics".into(),
            engine: Engine::MySql,
            host: "analytics.acme.io".into(),
            port: 3306,
            database: "warehouse".into(),
            user: "analyst".into(),
            environment: Environment::Production,
            safe_mode: SafeMode::ReadOnly,
            ssl: true,
            ssh: None,
            group: "Acme".into(),
            last_used: "last week".into(),
            outcome: ConnectOutcome::Unreachable,
        },
        Connection {
            name: "Production".into(),
            engine: Engine::Postgres,
            host: "prod-db-1.acme.io".into(),
            port: 5432,
            database: "acme_prod".into(),
            user: "acme_ops".into(),
            environment: Environment::Production,
            safe_mode: SafeMode::Safe,
            ssl: true,
            ssh: Some("bastion.acme.io".into()),
            group: "Acme".into(),
            last_used: "1 hour ago".into(),
            outcome: ConnectOutcome::Ok,
        },
        Connection {
            name: "Scratch".into(),
            engine: Engine::Sqlite,
            host: "~/scratch.db".into(),
            port: 0,
            database: "scratch".into(),
            user: String::new(),
            environment: Environment::Local,
            safe_mode: SafeMode::Silent,
            ssl: false,
            ssh: None,
            group: "Personal".into(),
            last_used: "never".into(),
            outcome: ConnectOutcome::Ok,
        },
    ]
}

/// Deterministic pseudo-random stream (`SplitMix64`).
#[derive(Clone)]
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len() as u64) as usize]
    }

    fn chance(&mut self, pct: u64) -> bool {
        self.below(100) < pct
    }
}

const FIRST: &[&str] = &[
    "Mira", "Jonas", "Ana", "Kai", "Sofia", "Lucas", "Elena", "Omar", "Priya", "Tomas", "Ingrid",
    "Diego", "Hana", "Felix", "Nadia", "Ravi", "Greta", "Mateo", "Yuki", "Leon",
];

const LAST: &[&str] = &[
    "Okafor", "Weber", "Costa", "Tanaka", "Rossi", "Nguyen", "Petrov", "Haddad", "Iyer", "Novak",
    "Lindqvist", "Alvarez", "Sato", "Brandt", "Karim", "Mehta", "Berg", "Silva", "Mori",
    "Fischer",
];

const ORGS: &[&str] = &[
    "Northwind Labs",
    "Lumen Retail",
    "Halden & Co",
    "Bluefin Logistics",
    "Orbit Studio",
    "Kestrel Health",
    "Fjord Analytics",
    "Tessellate",
    "Juniper Home",
    "Meridian Foods",
];

const DOMAINS: &[&str] = &[
    "northwind.io",
    "lumen.shop",
    "halden.co",
    "bluefin.dev",
    "orbit.studio",
    "kestrel.health",
    "fjord.ai",
    "tessellate.app",
    "juniper.home",
    "meridian.foods",
];

const PRODUCTS: &[&str] = &[
    "Standing desk 140",
    "Ergo chair",
    "Monitor arm",
    "USB-C dock",
    "Mechanical keyboard",
    "Desk lamp",
    "Cable tray",
    "Laptop stand",
    "Webcam 4K",
    "Headset Pro",
    "Whiteboard 90",
    "Footrest",
];

const STREETS: &[&str] = &[
    "Hauptstrasse",
    "Rue de Rivoli",
    "Market St",
    "Via Roma",
    "Calle Mayor",
    "Kungsgatan",
    "Nevsky Ave",
    "Queen St",
];

const CITIES: &[&str] = &[
    "Berlin",
    "Paris",
    "San Francisco",
    "Milan",
    "Madrid",
    "Stockholm",
    "Lisbon",
    "Toronto",
];

const NOTES: &[&str] = &[
    "Leave at reception; building closes 18:00.",
    "Customer asked for an invoice with VAT number DE812345678, please attach it to the shipment confirmation email.",
    "Gift wrap, no price on the slip.",
    "Second attempt after failed card. Confirmed by phone with the account owner, who asked us to hold the parcel at the depot until Monday.",
    "Replace damaged unit from order #10442.",
];

fn uuid(r: &mut Rng) -> String {
    let a = r.next();
    let b = r.next();
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        a & 0xffff_ffff,
        (a >> 32) & 0xffff,
        (a >> 48) & 0xfff,
        0x8000 | ((b >> 48) & 0x3fff),
        b & 0xffff_ffff_ffff
    )
}

fn timestamp(r: &mut Rng, year: i32) -> String {
    let month = 1 + r.below(12);
    let day = 1 + r.below(28);
    format!(
        "{year}-{month:02}-{day:02} {:02}:{:02}:{:02}+00",
        r.below(24),
        r.below(60),
        r.below(60)
    )
}

fn date(r: &mut Rng, year: i32) -> String {
    format!("{year}-{:02}-{:02}", 1 + r.below(12), 1 + r.below(28))
}

/// Generate deterministic rows for a table. `n` rows starting at `offset`.
pub fn rows(table: &Table, offset: usize, n: usize) -> Vec<Vec<Value>> {
    let mut out = Vec::with_capacity(n);
    for i in offset..offset + n {
        let mut r =
            Rng((i as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ (table.name.len() as u64) << 40);
        let mut row = Vec::with_capacity(table.columns.len());
        let first = r.pick(FIRST);
        let last = r.pick(LAST);
        let org_i = r.below(ORGS.len() as u64) as usize;
        for c in &table.columns {
            let v = match (c.name.as_str(), c.ty) {
                (_, ColType::Uuid) if c.primary => Value::Text(uuid(&mut r)),
                (_, ColType::Uuid) => {
                    if c.nullable && r.chance(20) {
                        Value::Null
                    } else {
                        Value::Text(uuid(&mut r))
                    }
                }
                ("order_number", _) => Value::Int(10_000 + i as i64),
                ("id", ColType::Int) => Value::Int(i as i64 + 1),
                ("email" | "billing_email", _) => Value::Text(format!(
                    "{}.{}@{}",
                    first.to_lowercase(),
                    last.to_lowercase(),
                    DOMAINS[org_i]
                )),
                ("full_name", _) => Value::Text(format!("{first} {last}")),
                ("name", _) if table.name == "organizations" => Value::Text(ORGS[org_i].into()),
                ("name", _) if table.name == "products" => Value::Text(r.pick(PRODUCTS).into()),
                ("slug", _) => Value::Text(
                    ORGS[org_i]
                        .to_lowercase()
                        .replace([' ', '&'], "-")
                        .replace("--", "-"),
                ),
                ("sku", _) => Value::Text(format!("SKU-{:05}", 1000 + i)),
                ("phone", _) => {
                    if r.chance(35) {
                        Value::Null
                    } else {
                        Value::Text(format!(
                            "+{} {} {:03} {:04}",
                            1 + r.below(48),
                            100 + r.below(900),
                            r.below(1000),
                            r.below(10000)
                        ))
                    }
                }
                ("table_name", _) => Value::Text(
                    r.pick(&["orders", "customers", "payments", "subscriptions"])
                        .into(),
                ),
                ("changed_by", _) => Value::Text(format!(
                    "{}@acme.io",
                    r.pick(&["mira", "jonas", "ana", "system", "deploy-bot"])
                )),
                ("ip", _) => Value::Text(format!(
                    "{}.{}.{}.{}",
                    10 + r.below(200),
                    r.below(255),
                    r.below(255),
                    1 + r.below(254)
                )),
                ("user_agent", _) => {
                    if r.chance(15) {
                        Value::Null
                    } else {
                        Value::Text(r.pick(&["Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) AppleWebKit/605.1.15 Safari/605.1.15", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_4 like Mac OS X) Mobile/15E148", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/124.0"]).into())
                    }
                }
                ("provider_ref", _) => Value::Text(format!("ch_{:016x}", r.next())),
                ("failure_reason", _) => {
                    if r.chance(80) {
                        Value::Null
                    } else {
                        Value::Text(
                            r.pick(&[
                                "insufficient_funds",
                                "card_declined",
                                "expired_card",
                                "3ds_failed",
                            ])
                            .into(),
                        )
                    }
                }
                ("description" | "notes", _) => {
                    if r.chance(45) {
                        Value::Null
                    } else {
                        Value::Text(r.pick(NOTES).into())
                    }
                }
                ("currency", _) => Value::Text(r.pick(&["USD", "USD", "EUR", "GBP"]).into()),
                (_, ColType::Enum) => Value::Text(r.pick(&c.enum_values).into()),
                ("seats", _) => Value::Int(1 + r.below(120) as i64),
                ("quantity", _) => Value::Int(1 + r.below(6) as i64),
                ("orders", _) => Value::Int(200 + r.below(900) as i64),
                ("customers" | "month_offset", _) => Value::Int(r.below(1000) as i64),
                (_, ColType::Int) => Value::Int(r.below(10_000) as i64),
                ("retained_pct", _) => Value::Num(20.0 + r.below(7500) as f64 / 100.0),
                (_, ColType::Numeric) => {
                    if c.nullable && r.chance(60) {
                        Value::Null
                    } else {
                        Value::Num((500 + r.below(250_000)) as f64 / 100.0)
                    }
                }
                ("succeeded", _) => Value::Bool(r.chance(88)),
                ("is_active", _) => Value::Bool(r.chance(85)),
                ("marketing_opt_in" | "is_gift", _) => Value::Bool(r.chance(20)),
                (_, ColType::Bool) => Value::Bool(r.chance(50)),
                ("shipping_address", _) => {
                    if r.chance(10) {
                        Value::Null
                    } else {
                        Value::Json(format!(
                            "{{\"line1\": \"{} {}\", \"city\": \"{}\", \"postal_code\": \"{:05}\", \"country\": \"{}\"}}",
                            1 + r.below(200),
                            r.pick(STREETS),
                            r.pick(CITIES),
                            r.below(99999),
                            r.pick(&["DE", "FR", "US", "IT", "ES", "SE"])
                        ))
                    }
                }
                ("settings", _) => Value::Json(format!(
                    "{{\"sso\": {}, \"retention_days\": {}, \"features\": [\"exports\"{}]}}",
                    r.chance(30),
                    30 * (1 + r.below(12)),
                    if r.chance(50) { ", \"audit\"" } else { "" }
                )),
                ("metadata", _) => {
                    if r.chance(50) {
                        Value::Null
                    } else {
                        Value::Json(format!(
                            "{{\"source\": \"{}\", \"utm_campaign\": \"{}\"}}",
                            r.pick(&["web", "ios", "android", "import"]),
                            r.pick(&["spring-24", "launch", "referral", "none"])
                        ))
                    }
                }
                ("attributes", _) => Value::Json(format!(
                    "{{\"color\": \"{}\", \"weight_kg\": {:.1}}}",
                    r.pick(&["black", "white", "walnut", "graphite"]),
                    r.below(300) as f64 / 10.0
                )),
                ("properties", _) => {
                    if r.chance(30) {
                        Value::Null
                    } else {
                        Value::Json(format!(
                            "{{\"path\": \"/{}\", \"referrer\": \"{}\"}}",
                            r.pick(&["", "pricing", "docs", "cart", "checkout"]),
                            r.pick(&["google", "direct", "newsletter", "twitter"])
                        ))
                    }
                }
                (_, ColType::Json) => Value::Null,
                ("created_at" | "occurred_at" | "changed_at" | "attempted_at", _) => {
                    let year = 2024 + r.below(2) as i32;
                    Value::Text(timestamp(&mut r, year))
                }
                (_, ColType::Timestamp) => {
                    if c.nullable && r.chance(40) {
                        Value::Null
                    } else {
                        Value::Text(timestamp(&mut r, 2025))
                    }
                }
                (_, ColType::Date) => {
                    if c.nullable && r.chance(50) {
                        Value::Null
                    } else {
                        Value::Text(date(&mut r, 2025))
                    }
                }
                (_, ColType::Text) => Value::Text(format!("{first} {last}")),
            };
            row.push(v);
        }
        out.push(row);
    }
    out
}
