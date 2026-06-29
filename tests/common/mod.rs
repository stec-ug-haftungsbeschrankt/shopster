use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

pub async fn test_harness<F, Fut>(test_code: F)
where
    F: FnOnce(String, String) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let tenet_node = Postgres::default().start().await.expect("Unable to create tenet container");
    let tenet_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", tenet_node.get_host_port_ipv4(5432).await.unwrap());

    let shopster_node = Postgres::default().start().await.expect("Unable to create shopster container");
    let shopster_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/stec_shopster_test", shopster_node.get_host_port_ipv4(5432).await.unwrap());

    test_code(tenet_connection_string, shopster_connection_string).await;

    shopster_node.stop().await.expect("Failed to stop shopster");
    tenet_node.stop().await.expect("Failed to stop tenet");
}

pub async fn test_harness_two_tenants<F, Fut>(test_code: F)
where
    F: FnOnce(String, String, String) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let tenet_node = Postgres::default().start().await.expect("Unable to create tenet container");
    let tenet_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", tenet_node.get_host_port_ipv4(5432).await.unwrap());

    let shopster_node1 = Postgres::default().start().await.expect("Unable to create shopster container 1");
    let shopster_connection_string1 = format!("postgres://postgres:postgres@127.0.0.1:{}/stec_shopster_test1", shopster_node1.get_host_port_ipv4(5432).await.unwrap());

    let shopster_node2 = Postgres::default().start().await.expect("Unable to create shopster container 2");
    let shopster_connection_string2 = format!("postgres://postgres:postgres@127.0.0.1:{}/stec_shopster_test2", shopster_node2.get_host_port_ipv4(5432).await.unwrap());

    test_code(tenet_connection_string, shopster_connection_string1, shopster_connection_string2).await;

    shopster_node2.stop().await.expect("Failed to stop shopster tenant 2");
    shopster_node1.stop().await.expect("Failed to stop shopster tenant 1");
    tenet_node.stop().await.expect("Failed to stop tenet");
}
