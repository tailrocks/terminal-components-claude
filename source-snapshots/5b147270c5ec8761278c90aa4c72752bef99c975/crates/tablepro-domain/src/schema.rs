//! Catalog, table, column, index, and constraint domain types.

/// Column storage type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColType {
    /// UUID-like identifier.
    Uuid,
    /// Text value.
    Text,
    /// Signed integer.
    Int,
    /// Decimal number.
    Numeric,
    /// Boolean value.
    Bool,
    /// Timestamp value.
    Timestamp,
    /// Calendar date.
    Date,
    /// JSON value.
    Json,
    /// Enumerated text value.
    Enum,
}

impl ColType {
    pub fn sql(self) -> &'static str {
        match self {
            ColType::Uuid => "uuid",
            ColType::Text | ColType::Enum => "text",
            ColType::Int => "integer",
            ColType::Numeric => "numeric(12,2)",
            ColType::Bool => "boolean",
            ColType::Timestamp => "timestamptz",
            ColType::Date => "date",
            ColType::Json => "jsonb",
        }
    }
}

/// Column metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub ty: ColType,
    pub nullable: bool,
    pub default: Option<String>,
    pub primary: bool,
    pub references: Option<(String, String)>,
    pub enum_values: Vec<&'static str>,
    pub generated: bool,
}

/// Index definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub method: &'static str,
}

/// Table constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    pub name: String,
    pub kind: &'static str,
    pub definition: String,
}

/// Catalog object kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    Table,
    View,
    Function,
    Sequence,
}

/// Catalog table metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub schema: String,
    pub name: String,
    pub kind: ObjectKind,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
    pub constraints: Vec<Constraint>,
    pub triggers: Vec<String>,
    pub row_count: usize,
    pub comment: Option<String>,
}

impl Table {
    pub fn qualified(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }

    pub fn column(&self, name: &str) -> Option<&Column> {
        self.columns
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
    }

    pub fn primary_key(&self) -> Vec<&Column> {
        self.columns.iter().filter(|c| c.primary).collect()
    }
}

/// Database catalog metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    pub database: String,
    pub schemas: Vec<String>,
    pub tables: Vec<Table>,
}

impl Catalog {
    pub fn new(database: impl Into<String>, schemas: Vec<String>, tables: Vec<Table>) -> Self {
        Self {
            database: database.into(),
            schemas,
            tables,
        }
    }

    pub fn find(&self, schema: Option<&str>, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| {
            if let Some(s) = schema {
                t.schema.eq_ignore_ascii_case(s) && t.name.eq_ignore_ascii_case(name)
            } else {
                t.name.eq_ignore_ascii_case(name)
            }
        })
    }

    pub fn table(&self, schema: &str, name: &str) -> Option<&Table> {
        self.find(Some(schema), name)
    }

    pub fn tables_in(&self, schema: &str, kind: ObjectKind) -> impl Iterator<Item = &Table> {
        self.tables
            .iter()
            .filter(move |t| t.schema == schema && t.kind == kind)
    }

    /// Build the stable `acme_prod` fixture catalog.
    pub fn acme_prod() -> Self {
        let created = || dflt(col("created_at", ColType::Timestamp), "now()");
        let updated = || nullable(col("updated_at", ColType::Timestamp));
        let tables = vec![
            Table {
                schema: "public".into(),
                name: "organizations".into(),
                kind: ObjectKind::Table,
                columns: vec![
                    pk("id"),
                    col("name", ColType::Text),
                    col("slug", ColType::Text),
                    enum_col("plan", vec!["free", "starter", "team", "enterprise"]),
                    nullable(col("billing_email", ColType::Text)),
                    dflt(col("seats", ColType::Int), "5"),
                    nullable(col("settings", ColType::Json)),
                    created(),
                    updated(),
                ],
                indexes: vec![
                    idx("organizations_pkey", &["id"], true),
                    idx("organizations_slug_key", &["slug"], true),
                ],
                constraints: vec![
                    Constraint { name: "organizations_pkey".into(), kind: "PRIMARY KEY", definition: "(id)".into() },
                    Constraint { name: "organizations_slug_key".into(), kind: "UNIQUE", definition: "(slug)".into() },
                    Constraint { name: "organizations_seats_check".into(), kind: "CHECK", definition: "(seats > 0)".into() },
                ],
                triggers: vec!["set_updated_at BEFORE UPDATE".into()],
                row_count: 1_240,
                comment: Some("Tenant accounts".into()),
            },
            Table {
                schema: "public".into(),
                name: "customers".into(),
                kind: ObjectKind::Table,
                columns: vec![
                    pk("id"),
                    fk("organization_id", "organizations"),
                    col("email", ColType::Text),
                    col("full_name", ColType::Text),
                    nullable(col("phone", ColType::Text)),
                    dflt(col("is_active", ColType::Bool), "true"),
                    dflt(col("marketing_opt_in", ColType::Bool), "false"),
                    nullable(col("last_login_at", ColType::Timestamp)),
                    nullable(col("metadata", ColType::Json)),
                    created(),
                    updated(),
                ],
                indexes: vec![
                    idx("customers_pkey", &["id"], true),
                    idx("customers_email_key", &["email"], true),
                    idx("customers_org_idx", &["organization_id"], false),
                ],
                constraints: vec![
                    Constraint { name: "customers_pkey".into(), kind: "PRIMARY KEY", definition: "(id)".into() },
                    Constraint { name: "customers_email_key".into(), kind: "UNIQUE", definition: "(email)".into() },
                    Constraint { name: "customers_organization_id_fkey".into(), kind: "FOREIGN KEY", definition: "(organization_id) REFERENCES organizations(id) ON DELETE CASCADE".into() },
                ],
                triggers: vec!["set_updated_at BEFORE UPDATE".into(), "audit_customers AFTER INSERT OR UPDATE OR DELETE".into()],
                row_count: 48_912,
                comment: None,
            },
            Table {
                schema: "public".into(),
                name: "products".into(),
                kind: ObjectKind::Table,
                columns: vec![
                    pk("id"),
                    col("sku", ColType::Text),
                    col("name", ColType::Text),
                    nullable(col("description", ColType::Text)),
                    col("unit_price", ColType::Numeric),
                    dflt(col("currency", ColType::Text), "'USD'"),
                    dflt(col("is_active", ColType::Bool), "true"),
                    nullable(col("attributes", ColType::Json)),
                    created(),
                ],
                indexes: vec![idx("products_pkey", &["id"], true), idx("products_sku_key", &["sku"], true)],
                constraints: vec![
                    Constraint { name: "products_pkey".into(), kind: "PRIMARY KEY", definition: "(id)".into() },
                    Constraint { name: "products_sku_key".into(), kind: "UNIQUE", definition: "(sku)".into() },
                    Constraint { name: "products_unit_price_check".into(), kind: "CHECK", definition: "(unit_price >= 0)".into() },
                ],
                triggers: vec![],
                row_count: 312,
                comment: None,
            },
            Table {
                schema: "public".into(),
                name: "orders".into(),
                kind: ObjectKind::Table,
                columns: vec![
                    pk("id"),
                    Column { generated: true, default: Some("nextval('orders_number_seq')".into()), ..col("order_number", ColType::Int) },
                    fk("customer_id", "customers"),
                    fk("organization_id", "organizations"),
                    dflt(enum_col("status", vec!["pending", "paid", "shipped", "delivered", "cancelled", "refunded"]), "'pending'"),
                    col("total_amount", ColType::Numeric),
                    dflt(col("currency", ColType::Text), "'USD'"),
                    nullable(col("shipping_address", ColType::Json)),
                    nullable(col("notes", ColType::Text)),
                    nullable(col("shipped_at", ColType::Timestamp)),
                    nullable(col("delivered_at", ColType::Date)),
                    dflt(col("is_gift", ColType::Bool), "false"),
                    created(),
                    updated(),
                ],
                indexes: vec![
                    idx("orders_pkey", &["id"], true),
                    idx("orders_order_number_key", &["order_number"], true),
                    idx("orders_customer_idx", &["customer_id"], false),
                    idx("orders_status_created_idx", &["status", "created_at"], false),
                ],
                constraints: vec![
                    Constraint { name: "orders_pkey".into(), kind: "PRIMARY KEY", definition: "(id)".into() },
                    Constraint { name: "orders_customer_id_fkey".into(), kind: "FOREIGN KEY", definition: "(customer_id) REFERENCES customers(id)".into() },
                    Constraint { name: "orders_organization_id_fkey".into(), kind: "FOREIGN KEY", definition: "(organization_id) REFERENCES organizations(id)".into() },
                    Constraint { name: "orders_status_check".into(), kind: "CHECK", definition: "(status IN ('pending','paid','shipped','delivered','cancelled','refunded'))".into() },
                    Constraint { name: "orders_total_amount_check".into(), kind: "CHECK", definition: "(total_amount >= 0)".into() },
                ],
                triggers: vec!["set_updated_at BEFORE UPDATE".into(), "orders_audit AFTER UPDATE".into()],
                row_count: 1_203_338,
                comment: Some("One row per checkout".into()),
            },
            Table {
                schema: "public".into(),
                name: "order_items".into(),
                kind: ObjectKind::Table,
                columns: vec![
                    pk("id"),
                    fk("order_id", "orders"),
                    fk("product_id", "products"),
                    col("quantity", ColType::Int),
                    col("unit_price", ColType::Numeric),
                    nullable(col("discount", ColType::Numeric)),
                    created(),
                ],
                indexes: vec![idx("order_items_pkey", &["id"], true), idx("order_items_order_idx", &["order_id"], false)],
                constraints: vec![
                    Constraint { name: "order_items_pkey".into(), kind: "PRIMARY KEY", definition: "(id)".into() },
                    Constraint { name: "order_items_order_id_fkey".into(), kind: "FOREIGN KEY", definition: "(order_id) REFERENCES orders(id) ON DELETE CASCADE".into() },
                    Constraint { name: "order_items_product_id_fkey".into(), kind: "FOREIGN KEY", definition: "(product_id) REFERENCES products(id)".into() },
                    Constraint { name: "order_items_quantity_check".into(), kind: "CHECK", definition: "(quantity > 0)".into() },
                ],
                triggers: vec![],
                row_count: 3_811_020,
                comment: None,
            },
            Table {
                schema: "public".into(),
                name: "payments".into(),
                kind: ObjectKind::Table,
                columns: vec![
                    pk("id"),
                    fk("order_id", "orders"),
                    enum_col("provider", vec!["stripe", "paypal", "wire", "apple_pay"]),
                    nullable(col("provider_ref", ColType::Text)),
                    col("amount", ColType::Numeric),
                    dflt(col("currency", ColType::Text), "'USD'"),
                    enum_col("status", vec!["authorized", "captured", "failed", "refunded"]),
                    nullable(col("failure_reason", ColType::Text)),
                    nullable(col("captured_at", ColType::Timestamp)),
                    created(),
                ],
                indexes: vec![idx("payments_pkey", &["id"], true), idx("payments_order_idx", &["order_id"], false), idx("payments_provider_ref_idx", &["provider", "provider_ref"], false)],
                constraints: vec![
                    Constraint { name: "payments_pkey".into(), kind: "PRIMARY KEY", definition: "(id)".into() },
                    Constraint { name: "payments_order_id_fkey".into(), kind: "FOREIGN KEY", definition: "(order_id) REFERENCES orders(id)".into() },
                ],
                triggers: vec![],
                row_count: 1_180_442,
                comment: None,
            },
            Table {
                schema: "public".into(),
                name: "subscriptions".into(),
                kind: ObjectKind::Table,
                columns: vec![
                    pk("id"),
                    fk("organization_id", "organizations"),
                    enum_col("plan", vec!["starter", "team", "enterprise"]),
                    enum_col("status", vec!["trialing", "active", "past_due", "cancelled"]),
                    col("seats", ColType::Int),
                    col("mrr", ColType::Numeric),
                    col("current_period_end", ColType::Date),
                    nullable(col("cancelled_at", ColType::Timestamp)),
                    created(),
                    updated(),
                ],
                indexes: vec![idx("subscriptions_pkey", &["id"], true), idx("subscriptions_org_idx", &["organization_id"], false)],
                constraints: vec![
                    Constraint { name: "subscriptions_pkey".into(), kind: "PRIMARY KEY", definition: "(id)".into() },
                    Constraint { name: "subscriptions_organization_id_fkey".into(), kind: "FOREIGN KEY", definition: "(organization_id) REFERENCES organizations(id)".into() },
                ],
                triggers: vec!["set_updated_at BEFORE UPDATE".into()],
                row_count: 1_102,
                comment: None,
            },
            Table {
                schema: "public".into(),
                name: "active_customers".into(),
                kind: ObjectKind::View,
                columns: vec![col("id", ColType::Uuid), col("email", ColType::Text), col("full_name", ColType::Text), col("organization", ColType::Text)],
                indexes: vec![],
                constraints: vec![],
                triggers: vec![],
                row_count: 41_207,
                comment: Some("customers WHERE is_active".into()),
            },
            Table {
                schema: "public".into(),
                name: "order_totals_by_day".into(),
                kind: ObjectKind::View,
                columns: vec![col("day", ColType::Date), col("orders", ColType::Int), col("revenue", ColType::Numeric)],
                indexes: vec![],
                constraints: vec![],
                triggers: vec![],
                row_count: 1_460,
                comment: None,
            },
            Table {
                schema: "analytics".into(),
                name: "events".into(),
                kind: ObjectKind::Table,
                columns: vec![
                    pk("id"),
                    nullable(fk("customer_id", "customers")),
                    enum_col("event_type", vec!["page_view", "add_to_cart", "checkout", "signup", "login", "search"]),
                    nullable(col("session_id", ColType::Uuid)),
                    nullable(col("properties", ColType::Json)),
                    nullable(col("user_agent", ColType::Text)),
                    col("occurred_at", ColType::Timestamp),
                ],
                indexes: vec![idx("events_pkey", &["id"], true), idx("events_occurred_idx", &["occurred_at"], false), idx("events_type_idx", &["event_type"], false)],
                constraints: vec![Constraint { name: "events_pkey".into(), kind: "PRIMARY KEY", definition: "(id)".into() }],
                triggers: vec![],
                row_count: 98_442_010,
                comment: Some("Product analytics, partitioned by month".into()),
            },
            Table {
                schema: "analytics".into(),
                name: "daily_revenue".into(),
                kind: ObjectKind::Table,
                columns: vec![col("day", ColType::Date), col("orders", ColType::Int), col("revenue", ColType::Numeric), col("refunds", ColType::Numeric), col("net", ColType::Numeric)],
                indexes: vec![idx("daily_revenue_day_key", &["day"], true)],
                constraints: vec![],
                triggers: vec![],
                row_count: 1_460,
                comment: Some("Materialized nightly".into()),
            },
            Table {
                schema: "analytics".into(),
                name: "cohort_retention".into(),
                kind: ObjectKind::View,
                columns: vec![col("cohort_month", ColType::Date), col("month_offset", ColType::Int), col("customers", ColType::Int), col("retained_pct", ColType::Numeric)],
                indexes: vec![],
                constraints: vec![],
                triggers: vec![],
                row_count: 288,
                comment: None,
            },
            Table {
                schema: "audit".into(),
                name: "change_log".into(),
                kind: ObjectKind::Table,
                columns: vec![
                    Column { generated: true, default: Some("identity".into()), ..pk("id") },
                    col("table_name", ColType::Text),
                    col("row_id", ColType::Uuid),
                    enum_col("operation", vec!["INSERT", "UPDATE", "DELETE"]),
                    nullable(col("changed_by", ColType::Text)),
                    nullable(col("old_values", ColType::Json)),
                    nullable(col("new_values", ColType::Json)),
                    col("changed_at", ColType::Timestamp),
                ],
                indexes: vec![idx("change_log_pkey", &["id"], true), idx("change_log_table_row_idx", &["table_name", "row_id"], false)],
                constraints: vec![Constraint { name: "change_log_pkey".into(), kind: "PRIMARY KEY", definition: "(id)".into() }],
                triggers: vec![],
                row_count: 12_004_118,
                comment: None,
            },
            Table {
                schema: "audit".into(),
                name: "login_attempts".into(),
                kind: ObjectKind::Table,
                columns: vec![
                    pk("id"),
                    col("email", ColType::Text),
                    col("succeeded", ColType::Bool),
                    nullable(col("ip", ColType::Text)),
                    col("attempted_at", ColType::Timestamp),
                ],
                indexes: vec![idx("login_attempts_pkey", &["id"], true), idx("login_attempts_email_idx", &["email"], false)],
                constraints: vec![],
                triggers: vec![],
                row_count: 3_301_002,
                comment: None,
            },
            Table {
                schema: "public".into(),
                name: "set_updated_at()".into(),
                kind: ObjectKind::Function,
                columns: vec![],
                indexes: vec![],
                constraints: vec![],
                triggers: vec![],
                row_count: 0,
                comment: Some("trigger function, plpgsql".into()),
            },
            Table {
                schema: "public".into(),
                name: "orders_number_seq".into(),
                kind: ObjectKind::Sequence,
                columns: vec![],
                indexes: vec![],
                constraints: vec![],
                triggers: vec![],
                row_count: 0,
                comment: None,
            },
        ];
        Self {
            database: "acme_prod".into(),
            schemas: vec!["public".into(), "analytics".into(), "audit".into()],
            tables,
        }
    }
}

fn col(name: &str, ty: ColType) -> Column {
    Column {
        name: name.into(),
        ty,
        nullable: false,
        default: None,
        primary: false,
        references: None,
        enum_values: vec![],
        generated: false,
    }
}

fn pk(name: &str) -> Column {
    Column {
        primary: true,
        default: Some("gen_random_uuid()".into()),
        ..col(name, ColType::Uuid)
    }
}

fn fk(name: &str, table: &str) -> Column {
    Column {
        references: Some((table.into(), "id".into())),
        ..col(name, ColType::Uuid)
    }
}

fn nullable(c: Column) -> Column {
    Column {
        nullable: true,
        ..c
    }
}

fn dflt(c: Column, d: &str) -> Column {
    Column {
        default: Some(d.into()),
        ..c
    }
}

fn enum_col(name: &str, values: Vec<&'static str>) -> Column {
    Column {
        enum_values: values,
        ..col(name, ColType::Enum)
    }
}

fn idx(name: &str, cols: &[&str], unique: bool) -> Index {
    Index {
        name: name.into(),
        columns: cols.iter().map(|c| (*c).into()).collect(),
        unique,
        method: "btree",
    }
}
