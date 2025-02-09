use std::fmt::Write;
use std::io;

use futures_util::{stream::FuturesUnordered, TryFutureExt, TryStreamExt};
use nanorand::{Rng, WyRand};
use tokio_postgres::{connect, types::ToSql, Client, NoTls, Statement};
use viz::{Error, IntoResponse, Response, StatusCode};

use crate::models::{Fortune, World};

/// Postgres Error
#[derive(Debug, thiserror::Error)]
pub enum PgError {
    #[error("connect to database was failed")]
    Connect,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Pg(#[from] tokio_postgres::Error),
}

impl From<PgError> for Error {
    fn from(e: PgError) -> Self {
        Error::Responder(e.into_response())
    }
}

impl IntoResponse for PgError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

/// Postgres interface
pub struct PgConnection {
    rng: WyRand,
    client: Client,
    world: Statement,
    fortune: Statement,
    updates: Vec<Statement>,
}

impl PgConnection {
    pub async fn connect(db_url: &str) -> PgConnection {
        let (client, conn) = connect(db_url, NoTls)
            .await
            .expect("can not connect to postgresql");

        // Spawn connection
        tokio::spawn(async move {
            if let Err(error) = conn.await {
                eprintln!("Connection error: {}", error);
            }
        });

        let fortune = client.prepare("SELECT * FROM fortune").await.unwrap();
        let mut updates = Vec::new();

        for num in 1..=500u16 {
            let mut pl = 1;
            let mut q = String::new();

            q.push_str("UPDATE world SET randomnumber = CASE id ");

            for _ in 1..=num {
                let _ = write!(q, "when ${} then ${} ", pl, pl + 1);
                pl += 2;
            }

            q.push_str("ELSE randomnumber END WHERE id IN (");

            for _ in 1..=num {
                let _ = write!(q, "${},", pl);
                pl += 1;
            }

            q.pop();
            q.push(')');

            updates.push(client.prepare(&q).await.unwrap());
        }

        let world = client
            .prepare("SELECT * FROM world WHERE id = $1")
            .await
            .unwrap();

        PgConnection {
            rng: WyRand::new(),
            world,
            client,
            fortune,
            updates,
        }
    }
}

impl PgConnection {
    async fn query_one_world(&self, id: i32) -> Result<World, PgError> {
        self.client
            .query_one(&self.world, &[&id])
            .await
            .map(|row| World {
                id: row.get(0),
                randomnumber: row.get(1),
            })
            .map_err(PgError::Pg)
    }

    pub async fn get_world(&self) -> Result<World, PgError> {
        let random_id = (self.rng.clone().generate::<u32>() % 10_000 + 1) as i32;
        self.query_one_world(random_id).await
    }

    pub async fn get_worlds(&self, num: u16) -> Result<Vec<World>, PgError> {
        let mut rng = self.rng.clone();
        (0..num)
            .map(|_| {
                let id = (rng.generate::<u32>() % 10_000 + 1) as i32;
                self.query_one_world(id)
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect()
            .await
    }

    pub async fn get_worlds_by_limit(&self, limit: i64) -> Result<Vec<World>, PgError> {
        self.client
            .query("SELECT * FROM world LIMIT $1", &[&limit])
            .await
            .map(|rows| {
                rows.iter()
                    .map(|row| World {
                        id: row.get(0),
                        randomnumber: row.get(1),
                    })
                    .collect()
            })
            .map_err(PgError::Pg)
    }

    pub async fn update(&self, num: u16) -> Result<Vec<World>, PgError> {
        let mut rng = self.rng.clone();

        let worlds: Vec<World> = (0..num)
            .map(|_| {
                let id = (rng.generate::<u32>() % 10_000 + 1) as i32;
                let rid = (rng.generate::<u32>() % 10_000 + 1) as i32;
                self.query_one_world(id).map_ok(move |mut world| {
                    world.randomnumber = rid;
                    world
                })
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect()
            .await?;

        let mut params: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(num as usize * 3);

        for w in &worlds {
            params.push(&w.id);
            params.push(&w.randomnumber);
        }

        for w in &worlds {
            params.push(&w.id);
        }

        let st = self.updates[(num as usize) - 1].clone();

        self.client.query(&st, &params[..]).await?;

        Ok(worlds)
    }

    pub async fn tell_fortune(&self) -> Result<Vec<Fortune>, PgError> {
        let mut items = Vec::with_capacity(32);

        items.push(Fortune {
            id: 0,
            message: "Additional fortune added at request time.".to_string(),
        });

        self.client
            .query(&self.fortune, &[])
            .await?
            .iter()
            .for_each(|row| {
                items.push(Fortune {
                    id: row.get(0),
                    message: row.get(1),
                })
            });

        items.sort_by(|it, next| it.message.cmp(&next.message));

        Ok(items)
    }
}
