use common::api::queue::get_work::QueueWork;
use sqlx::PgPool;

#[derive(sqlx::Type)]
#[sqlx(type_name = "job_type")]
enum JobType {
    #[sqlx(rename = "ping_server")]
    PingServer,
    #[sqlx(rename = "get_player")]
    GetPlayer,
}

pub async fn register_task(work: QueueWork, pool: &PgPool) -> Result<(), sqlx::Error> {
    let (job_type, value) = match work {
        QueueWork::PingServer(ip) => (JobType::PingServer, ip.to_string()),
        QueueWork::GetPlayer(uuid) => (JobType::GetPlayer, uuid),
    };

    sqlx::query!(
        r#"
    insert into pending_jobs (time, type, data)
    values (now(), $1, $2)
    "#,
        job_type as JobType,
        value
    )
    .execute(pool)
    .await
    .map(|_| ())
}

#[derive(sqlx::FromRow)]
struct DatabaseTask {
    r#type: JobType,
    data: String,
    id: i32,
}

pub async fn get_tasks(count: usize, pool: &PgPool) -> Result<Vec<QueueWork>, sqlx::Error> {
    let mut tasks = Vec::new();

    let rows = sqlx::query_as!(
        DatabaseTask,
        r#"
    select type as "type: _", data, id
    from pending_jobs
    order by time desc
    limit $1
    "#,
        Some(count as i64)
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let work = match row.r#type {
            JobType::PingServer => QueueWork::PingServer(row.data.parse().unwrap()),
            JobType::GetPlayer => QueueWork::GetPlayer(row.data),
        };

        sqlx::query!("delete from pending_jobs where id = $1", row.id)
            .execute(pool)
            .await?;

        tasks.push(work);
    }

    Ok(tasks)
}

pub async fn check_and_make_admin_key(pool: &PgPool) -> anyhow::Result<()> {
    let query = sqlx::query!("select count(*) from api_keys where admin = true")
        .fetch_one(pool)
        .await?;

    if query.count.is_none() || query.count == Some(0) {
        let key = uuid::Uuid::new_v4().to_string();

        sqlx::query!("insert into api_keys (key, admin) values ($1, true)", key)
            .execute(pool)
            .await?;

        log::info!("created admin key: {}", key);
    }

    Ok(())
}
