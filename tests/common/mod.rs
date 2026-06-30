use tokio::sync::OnceCell;
use uuid::Uuid;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

/// Holds the two Postgres containers (tenet + shopster) shared by every test
/// in this binary. Started once on first use and kept alive for the lifetime
/// of the process; individual tests get isolation via a fresh tenant/database
/// inside these containers rather than a container of their own.
struct SharedContainers {
    tenet_connection_string: String,
    shopster_host_port: u16,
    _tenet_node: ContainerAsync<Postgres>,
    _shopster_node: ContainerAsync<Postgres>,
}

static SHARED_CONTAINERS: OnceCell<SharedContainers> = OnceCell::const_new();

async fn shared_containers() -> &'static SharedContainers {
    SHARED_CONTAINERS.get_or_init(|| async {
        let tenet_node = Postgres::default().start().await.expect("Unable to create tenet container");
        let tenet_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", tenet_node.get_host_port_ipv4(5432).await.unwrap());

        let shopster_node = Postgres::default().start().await.expect("Unable to create shopster container");
        let shopster_host_port = shopster_node.get_host_port_ipv4(5432).await.unwrap();

        SharedContainers {
            tenet_connection_string,
            shopster_host_port,
            _tenet_node: tenet_node,
            _shopster_node: shopster_node,
        }
    }).await
}

fn unique_shopster_database_url(host_port: u16) -> String {
    format!("postgres://postgres:postgres@127.0.0.1:{}/stec_shopster_test_{}", host_port, Uuid::new_v4().simple())
}

pub async fn test_harness<F, Fut>(test_code: F)
where
    F: FnOnce(String, String) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let shared = shared_containers().await;
    let shopster_connection_string = unique_shopster_database_url(shared.shopster_host_port);

    test_code(shared.tenet_connection_string.clone(), shopster_connection_string).await;
}

pub async fn test_harness_two_tenants<F, Fut>(test_code: F)
where
    F: FnOnce(String, String, String) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let shared = shared_containers().await;
    let shopster_connection_string1 = unique_shopster_database_url(shared.shopster_host_port);
    let shopster_connection_string2 = unique_shopster_database_url(shared.shopster_host_port);

    test_code(shared.tenet_connection_string.clone(), shopster_connection_string1, shopster_connection_string2).await;
}
