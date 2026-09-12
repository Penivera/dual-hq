use async_graphql::{
    Context, EmptySubscription, Enum, InputObject, Object, Result, Schema, SimpleObject,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};

use crate::entities::{
    application,
    opportunity::{self, OpportunityStatus, OpportunityType},
    user::{self, UserRole},
    Application, Opportunity, User,
};

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct SeaOrmEntityMetadata {
    pub columns: Vec<SeaOrmColumnMetadata>,
    pub primary_key: Vec<String>,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct SeaOrmColumnMetadata {
    pub name: String,
    pub nullable: bool,
    #[graphql(name = "type")]
    pub col_type: Option<String>,
    #[graphql(name = "type_")]
    pub type_: SeaOrmColumnTypeMetadata,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct SeaOrmColumnTypeMetadata {
    pub primitive: Option<String>,
    pub array: Option<Box<SeaOrmArrayTypeMetadata>>,
    pub enumeration: Option<SeaOrmEnumTypeMetadata>,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct SeaOrmArrayTypeMetadata {
    pub array: Option<Box<SeaOrmArrayTypeMetadata>>,
    pub primitive: Option<String>,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct SeaOrmEnumTypeMetadata {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct PaginationInfo {
    pub current: u64,
    pub pages: u64,
    pub offset: u64,
    pub total: u64,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
#[graphql(rename_items = "UPPERCASE")]
pub enum OrderByEnum {
    ASC,
    DESC,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct StringFilterInput {
    pub eq: Option<String>,
    pub ne: Option<String>,
    pub contains: Option<String>,
    pub starts_with: Option<String>,
    pub ends_with: Option<String>,
    pub is_null: Option<bool>,
    pub is_not_null: Option<bool>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct IntFilterInput {
    pub eq: Option<i32>,
    pub ne: Option<i32>,
    pub gt: Option<i32>,
    pub gte: Option<i32>,
    pub lt: Option<i32>,
    pub lte: Option<i32>,
    pub is_null: Option<bool>,
    pub is_not_null: Option<bool>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct DateTimeFilterInput {
    pub eq: Option<String>,
    pub ne: Option<String>,
    pub gt: Option<String>,
    pub gte: Option<String>,
    pub lt: Option<String>,
    pub lte: Option<String>,
    pub between: Option<Vec<String>>,
    pub is_null: Option<bool>,
    pub is_not_null: Option<bool>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct PaginationPageInput {
    pub limit: Option<u64>,
    pub page: Option<u64>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct PaginationInput {
    pub page: Option<PaginationPageInput>,
}

// ---------------- User Types ----------------

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct UserGql {
    pub id: i32,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub is_verified: bool,
    pub created_at: String,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct UsersResult {
    pub nodes: Vec<UserGql>,
    pub pagination_info: PaginationInfo,
    pub total_count: u64,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct UserFilterInput {
    pub id: Option<IntFilterInput>,
    pub full_name: Option<StringFilterInput>,
    pub email: Option<StringFilterInput>,
    pub role: Option<StringFilterInput>,
    pub created_at: Option<DateTimeFilterInput>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct UserOrderByInput {
    pub id: Option<OrderByEnum>,
    pub full_name: Option<OrderByEnum>,
    pub email: Option<OrderByEnum>,
    pub role: Option<OrderByEnum>,
    pub created_at: Option<OrderByEnum>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct UserCreateInput {
    pub full_name: String,
    pub email: String,
    pub password: Option<String>,
    pub role: Option<String>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct UserUpdateInput {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
}

// ---------------- Opportunity Types ----------------

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct OpportunityGql {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub company: String,
    pub location: String,
    #[graphql(name = "type")]
    pub type_: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct OpportunitiesResult {
    pub nodes: Vec<OpportunityGql>,
    pub pagination_info: PaginationInfo,
    pub total_count: u64,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct OpportunityFilterInput {
    pub id: Option<IntFilterInput>,
    pub title: Option<StringFilterInput>,
    pub company: Option<StringFilterInput>,
    pub location: Option<StringFilterInput>,
    #[graphql(name = "type")]
    pub type_: Option<StringFilterInput>,
    pub status: Option<StringFilterInput>,
    pub created_at: Option<DateTimeFilterInput>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct OpportunityOrderByInput {
    pub id: Option<OrderByEnum>,
    pub title: Option<OrderByEnum>,
    pub company: Option<OrderByEnum>,
    pub location: Option<OrderByEnum>,
    #[graphql(name = "type")]
    pub type_: Option<OrderByEnum>,
    pub status: Option<OrderByEnum>,
    pub created_at: Option<OrderByEnum>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct OpportunityCreateInput {
    pub title: String,
    pub description: Option<String>,
    pub company: String,
    pub location: String,
    #[graphql(name = "type")]
    pub type_: Option<String>,
    pub status: Option<String>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct OpportunityUpdateInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    #[graphql(name = "type")]
    pub type_: Option<String>,
    pub status: Option<String>,
}

// ---------------- Application Types ----------------

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct ApplicationGql {
    pub id: i32,
    pub user_id: i32,
    pub opportunity_id: i32,
    pub cover_letter: Option<String>,
    pub status: String,
    pub applied_at: String,
    pub updated_at: String,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct ApplicationsResult {
    pub nodes: Vec<ApplicationGql>,
    pub pagination_info: PaginationInfo,
    pub total_count: u64,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct ApplicationFilterInput {
    pub id: Option<IntFilterInput>,
    pub user_id: Option<IntFilterInput>,
    pub opportunity_id: Option<IntFilterInput>,
    pub status: Option<StringFilterInput>,
    pub applied_at: Option<DateTimeFilterInput>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct ApplicationOrderByInput {
    pub id: Option<OrderByEnum>,
    pub user_id: Option<OrderByEnum>,
    pub opportunity_id: Option<OrderByEnum>,
    pub status: Option<OrderByEnum>,
    pub applied_at: Option<OrderByEnum>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct ApplicationCreateInput {
    pub user_id: i32,
    pub opportunity_id: i32,
    pub cover_letter: Option<String>,
    pub status: Option<String>,
}

#[derive(InputObject, Default)]
#[graphql(rename_fields = "snake_case")]
pub struct ApplicationUpdateInput {
    pub status: Option<String>,
    pub cover_letter: Option<String>,
}

// ---------------- Query Root ----------------

pub struct QueryRoot;

#[Object(rename_fields = "snake_case", rename_args = "snake_case")]
impl QueryRoot {
    #[graphql(name = "_sea_orm_entity_metadata")]
    async fn _sea_orm_entity_metadata(
        &self,
        table_name: String,
    ) -> Option<SeaOrmEntityMetadata> {
        let admin_cfg = crate::admin::get_cached_admin_config();
        let raw_table = admin_cfg.raw_tables.get(&table_name)?;

        let mut columns = Vec::new();
        for col_cfg in &raw_table.table.columns {
            let field_name = col_cfg.field.clone();
            let is_id = field_name == "id" || field_name.ends_with("_id");
            let is_datetime = field_name.contains("date")
                || field_name.contains("_at")
                || field_name.contains("time");
            let is_role = field_name == "role";
            let is_opp_type = field_name == "type";
            let is_app_status = field_name == "status" && table_name == "applications";
            let is_opp_status = field_name == "status" && table_name == "opportunities";

            let (col_type, primitive, enumeration) = if is_id {
                (Some("integer".to_string()), Some("Int".to_string()), None)
            } else if is_datetime {
                (Some("datetime".to_string()), Some("DateTime".to_string()), None)
            } else if is_role {
                (
                    Some("string".to_string()),
                    Some("String".to_string()),
                    Some(SeaOrmEnumTypeMetadata {
                        name: "UserRole".to_string(),
                        variants: vec!["admin".to_string(), "applicant".to_string()],
                    }),
                )
            } else if is_opp_type {
                (
                    Some("string".to_string()),
                    Some("String".to_string()),
                    Some(SeaOrmEnumTypeMetadata {
                        name: "OpportunityType".to_string(),
                        variants: vec!["internship".to_string(), "job".to_string()],
                    }),
                )
            } else if is_opp_status {
                (
                    Some("string".to_string()),
                    Some("String".to_string()),
                    Some(SeaOrmEnumTypeMetadata {
                        name: "OpportunityStatus".to_string(),
                        variants: vec!["open".to_string(), "closed".to_string()],
                    }),
                )
            } else if is_app_status {
                (
                    Some("string".to_string()),
                    Some("String".to_string()),
                    Some(SeaOrmEnumTypeMetadata {
                        name: "ApplicationStatus".to_string(),
                        variants: vec![
                            "pending".to_string(),
                            "accepted".to_string(),
                            "rejected".to_string(),
                            "withdrawn".to_string(),
                        ],
                    }),
                )
            } else {
                (Some("string".to_string()), Some("String".to_string()), None)
            };

            let nullable = field_name == "cover_letter";

            columns.push(SeaOrmColumnMetadata {
                name: field_name,
                nullable,
                col_type,
                type_: SeaOrmColumnTypeMetadata {
                    primitive,
                    array: None,
                    enumeration,
                },
            });
        }

        // Include any additional entity columns if all_columns is true and not hidden
        if raw_table.table.all_columns {
            let existing_names: std::collections::HashSet<String> =
                columns.iter().map(|c| c.name.clone()).collect();

            let candidates: &[(&str, &str, bool)] = match table_name.as_str() {
                "opportunities" => &[
                    ("description", "string", false),
                    ("updated_at", "datetime", false),
                ],
                "applications" => &[("updated_at", "datetime", false)],
                _ => &[],
            };

            for &(name, ctype, nullable) in candidates {
                if !existing_names.contains(name)
                    && !raw_table.table.hidden_columns.iter().any(|h| h == name)
                {
                    let (primitive, col_type) = if ctype == "datetime" {
                        (Some("DateTime".to_string()), Some("datetime".to_string()))
                    } else {
                        (Some("String".to_string()), Some("string".to_string()))
                    };
                    columns.push(SeaOrmColumnMetadata {
                        name: name.to_string(),
                        nullable,
                        col_type,
                        type_: SeaOrmColumnTypeMetadata {
                            primitive,
                            array: None,
                            enumeration: None,
                        },
                    });
                }
            }
        }

        Some(SeaOrmEntityMetadata {
            primary_key: vec!["id".to_string()],
            columns,
        })
    }

    #[graphql(name = "sea_orm_current_user_role_permissions")]
    async fn sea_orm_current_user_role_permissions(&self) -> Option<String> {
        None
    }

    async fn users(
        &self,
        ctx: &Context<'_>,
        filters: Option<UserFilterInput>,
        order_by: Option<UserOrderByInput>,
        pagination: Option<PaginationInput>,
    ) -> Result<UsersResult> {
        let db = ctx.data::<DatabaseConnection>()?;
        let mut query = User::find();

        if let Some(f) = filters {
            if let Some(id_f) = f.id {
                if let Some(eq) = id_f.eq {
                    query = query.filter(user::Column::Id.eq(eq));
                }
            }
            if let Some(name_f) = f.full_name {
                if let Some(contains) = name_f.contains {
                    query = query.filter(user::Column::FullName.contains(&contains));
                } else if let Some(eq) = name_f.eq {
                    query = query.filter(user::Column::FullName.eq(eq));
                }
            }
            if let Some(email_f) = f.email {
                if let Some(contains) = email_f.contains {
                    query = query.filter(user::Column::Email.contains(&contains));
                } else if let Some(eq) = email_f.eq {
                    query = query.filter(user::Column::Email.eq(eq));
                }
            }
            if let Some(role_f) = f.role {
                if let Some(eq) = role_f.eq {
                    let role = match eq.to_lowercase().as_str() {
                        "admin" => UserRole::Admin,
                        _ => UserRole::Applicant,
                    };
                    query = query.filter(user::Column::Role.eq(role));
                }
            }
        }

        if let Some(ord) = order_by {
            if let Some(o) = ord.id {
                query = match o {
                    OrderByEnum::ASC => query.order_by_asc(user::Column::Id),
                    OrderByEnum::DESC => query.order_by_desc(user::Column::Id),
                };
            } else if let Some(o) = ord.created_at {
                query = match o {
                    OrderByEnum::ASC => query.order_by_asc(user::Column::CreatedAt),
                    OrderByEnum::DESC => query.order_by_desc(user::Column::CreatedAt),
                };
            } else {
                query = query.order_by_asc(user::Column::Id);
            }
        } else {
            query = query.order_by_asc(user::Column::Id);
        }

        let total = query.clone().count(db).await?;

        let (limit, page_idx) = match pagination.and_then(|p| p.page) {
            Some(p) => (p.limit.unwrap_or(20), p.page.unwrap_or(0)),
            None => (20, 0),
        };

        let offset = page_idx * limit;
        let users = query.offset(offset).limit(limit).all(db).await?;

        let pages = if limit > 0 {
            total.div_ceil(limit)
        } else {
            1
        };

        let nodes = users
            .into_iter()
            .map(|u| UserGql {
                id: u.id,
                full_name: u.full_name,
                email: u.email,
                role: match u.role {
                    UserRole::Admin => "admin".to_string(),
                    UserRole::Applicant => "applicant".to_string(),
                },
                is_verified: u.is_verified,
                created_at: u.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
            .collect();

        Ok(UsersResult {
            nodes,
            pagination_info: PaginationInfo {
                current: page_idx + 1,
                pages,
                offset,
                total,
            },
            total_count: total,
        })
    }

    async fn opportunities(
        &self,
        ctx: &Context<'_>,
        filters: Option<OpportunityFilterInput>,
        order_by: Option<OpportunityOrderByInput>,
        pagination: Option<PaginationInput>,
    ) -> Result<OpportunitiesResult> {
        let db = ctx.data::<DatabaseConnection>()?;
        let mut query = Opportunity::find();

        if let Some(f) = filters {
            if let Some(id_f) = f.id {
                if let Some(eq) = id_f.eq {
                    query = query.filter(opportunity::Column::Id.eq(eq));
                }
            }
            if let Some(title_f) = f.title {
                if let Some(contains) = title_f.contains {
                    query = query.filter(opportunity::Column::Title.contains(&contains));
                } else if let Some(eq) = title_f.eq {
                    query = query.filter(opportunity::Column::Title.eq(eq));
                }
            }
            if let Some(company_f) = f.company {
                if let Some(contains) = company_f.contains {
                    query = query.filter(opportunity::Column::Company.contains(&contains));
                } else if let Some(eq) = company_f.eq {
                    query = query.filter(opportunity::Column::Company.eq(eq));
                }
            }
        }

        if let Some(ord) = order_by {
            if let Some(o) = ord.id {
                query = match o {
                    OrderByEnum::ASC => query.order_by_asc(opportunity::Column::Id),
                    OrderByEnum::DESC => query.order_by_desc(opportunity::Column::Id),
                };
            } else {
                query = query.order_by_desc(opportunity::Column::Id);
            }
        } else {
            query = query.order_by_desc(opportunity::Column::Id);
        }

        let total = query.clone().count(db).await?;

        let (limit, page_idx) = match pagination.and_then(|p| p.page) {
            Some(p) => (p.limit.unwrap_or(20), p.page.unwrap_or(0)),
            None => (20, 0),
        };

        let offset = page_idx * limit;
        let opps = query.offset(offset).limit(limit).all(db).await?;

        let pages = if limit > 0 {
            total.div_ceil(limit)
        } else {
            1
        };

        let nodes = opps
            .into_iter()
            .map(|o| OpportunityGql {
                id: o.id,
                title: o.title,
                description: o.description,
                company: o.company,
                location: o.location,
                type_: match o.type_ {
                    OpportunityType::Internship => "internship".to_string(),
                    OpportunityType::Job => "job".to_string(),
                },
                status: match o.status {
                    OpportunityStatus::Open => "open".to_string(),
                    OpportunityStatus::Closed => "closed".to_string(),
                },
                created_at: o.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: o.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
            .collect();

        Ok(OpportunitiesResult {
            nodes,
            pagination_info: PaginationInfo {
                current: page_idx + 1,
                pages,
                offset,
                total,
            },
            total_count: total,
        })
    }

    async fn applications(
        &self,
        ctx: &Context<'_>,
        filters: Option<ApplicationFilterInput>,
        order_by: Option<ApplicationOrderByInput>,
        pagination: Option<PaginationInput>,
    ) -> Result<ApplicationsResult> {
        let db = ctx.data::<DatabaseConnection>()?;
        let mut query = Application::find();

        if let Some(f) = filters {
            if let Some(id_f) = f.id {
                if let Some(eq) = id_f.eq {
                    query = query.filter(application::Column::Id.eq(eq));
                }
            }
            if let Some(u_f) = f.user_id {
                if let Some(eq) = u_f.eq {
                    query = query.filter(application::Column::UserId.eq(eq));
                }
            }
            if let Some(o_f) = f.opportunity_id {
                if let Some(eq) = o_f.eq {
                    query = query.filter(application::Column::OpportunityId.eq(eq));
                }
            }
        }

        if let Some(ord) = order_by {
            if let Some(o) = ord.id {
                query = match o {
                    OrderByEnum::ASC => query.order_by_asc(application::Column::Id),
                    OrderByEnum::DESC => query.order_by_desc(application::Column::Id),
                };
            } else {
                query = query.order_by_desc(application::Column::Id);
            }
        } else {
            query = query.order_by_desc(application::Column::Id);
        }

        let total = query.clone().count(db).await?;

        let (limit, page_idx) = match pagination.and_then(|p| p.page) {
            Some(p) => (p.limit.unwrap_or(20), p.page.unwrap_or(0)),
            None => (20, 0),
        };

        let offset = page_idx * limit;
        let apps = query.offset(offset).limit(limit).all(db).await?;

        let pages = if limit > 0 {
            total.div_ceil(limit)
        } else {
            1
        };

        let nodes = apps
            .into_iter()
            .map(|a| ApplicationGql {
                id: a.id,
                user_id: a.user_id,
                opportunity_id: a.opportunity_id,
                cover_letter: a.cover_letter,
                status: match a.status {
                    crate::entities::application::ApplicationStatus::Pending => {
                        "pending".to_string()
                    }
                    crate::entities::application::ApplicationStatus::Accepted => {
                        "accepted".to_string()
                    }
                    crate::entities::application::ApplicationStatus::Rejected => {
                        "rejected".to_string()
                    }
                    crate::entities::application::ApplicationStatus::Withdrawn => {
                        "withdrawn".to_string()
                    }
                },
                applied_at: a.applied_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: a.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
            .collect();

        Ok(ApplicationsResult {
            nodes,
            pagination_info: PaginationInfo {
                current: page_idx + 1,
                pages,
                offset,
                total,
            },
            total_count: total,
        })
    }
}

// ---------------- Mutation Root ----------------

pub struct MutationRoot;

#[Object(rename_fields = "snake_case", rename_args = "snake_case")]
impl MutationRoot {
    async fn users_create_one(
        &self,
        ctx: &Context<'_>,
        data: UserCreateInput,
    ) -> Result<UserGql> {
        let db = ctx.data::<DatabaseConnection>()?;
        let hashed_password = crate::handlers::auth::hash_password(
            data.password.as_deref().unwrap_or("changeme123"),
        )
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let role = match data.role.as_deref().unwrap_or("applicant").to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            _ => UserRole::Applicant,
        };

        let new_user = user::ActiveModel {
            full_name: Set(data.full_name),
            email: Set(data.email),
            hashed_password: Set(hashed_password),
            role: Set(role),
            created_at: Set(chrono::Utc::now().fixed_offset()),
            ..Default::default()
        };

        let inserted = new_user.insert(db).await?;
        Ok(UserGql {
            id: inserted.id,
            full_name: inserted.full_name,
            email: inserted.email,
            role: match inserted.role {
                UserRole::Admin => "admin".to_string(),
                UserRole::Applicant => "applicant".to_string(),
            },
            is_verified: inserted.is_verified,
            created_at: inserted.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    async fn users_update(
        &self,
        ctx: &Context<'_>,
        data: UserUpdateInput,
        filter: UserFilterInput,
    ) -> Result<Vec<UserGql>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let mut target_users = Vec::new();

        if let Some(id_f) = filter.id {
            if let Some(id) = id_f.eq {
                if let Some(u) = User::find_by_id(id).one(db).await? {
                    let mut active: user::ActiveModel = u.into();
                    if let Some(name) = data.full_name {
                        active.full_name = Set(name);
                    }
                    if let Some(email) = data.email {
                        active.email = Set(email);
                    }
                    if let Some(role) = data.role {
                        active.role = Set(match role.to_lowercase().as_str() {
                            "admin" => UserRole::Admin,
                            _ => UserRole::Applicant,
                        });
                    }
                    let updated = active.update(db).await?;
                    target_users.push(UserGql {
                        id: updated.id,
                        full_name: updated.full_name,
                        email: updated.email,
                        role: match updated.role {
                            UserRole::Admin => "admin".to_string(),
                            UserRole::Applicant => "applicant".to_string(),
                        },
                        is_verified: updated.is_verified,
                        created_at: updated.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    });
                }
            }
        }
        Ok(target_users)
    }

    async fn users_delete(
        &self,
        ctx: &Context<'_>,
        filter: UserFilterInput,
    ) -> Result<i32> {
        let db = ctx.data::<DatabaseConnection>()?;
        if let Some(id_f) = filter.id {
            if let Some(id) = id_f.eq {
                let res = User::delete_by_id(id).exec(db).await?;
                return Ok(res.rows_affected as i32);
            }
        }
        Ok(0)
    }

    async fn opportunities_delete(
        &self,
        ctx: &Context<'_>,
        filter: OpportunityFilterInput,
    ) -> Result<i32> {
        let db = ctx.data::<DatabaseConnection>()?;
        if let Some(id_f) = filter.id {
            if let Some(id) = id_f.eq {
                Application::delete_many()
                    .filter(application::Column::OpportunityId.eq(id))
                    .exec(db)
                    .await?;
                let res = Opportunity::delete_by_id(id).exec(db).await?;
                return Ok(res.rows_affected as i32);
            }
        }
        Ok(0)
    }

    async fn applications_delete(
        &self,
        ctx: &Context<'_>,
        filter: ApplicationFilterInput,
    ) -> Result<i32> {
        let db = ctx.data::<DatabaseConnection>()?;
        if let Some(id_f) = filter.id {
            if let Some(id) = id_f.eq {
                let res = Application::delete_by_id(id).exec(db).await?;
                return Ok(res.rows_affected as i32);
            }
        }
        Ok(0)
    }
}

pub type AdminSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

static ADMIN_SCHEMA: std::sync::OnceLock<AdminSchema> = std::sync::OnceLock::new();

pub fn get_admin_schema() -> &'static AdminSchema {
    ADMIN_SCHEMA.get_or_init(|| {
        Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
    })
}
