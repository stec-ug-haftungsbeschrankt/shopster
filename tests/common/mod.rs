
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::SyncRunner;

pub fn test_harness(test_code: impl Fn(String, String)) {
    let tenet_node = Postgres::default().start().expect("Unable to create to tenet container");
    let tenet_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", tenet_node.get_host_port_ipv4(5432).unwrap());

    let shopster_node = Postgres::default().start().expect("Unable to create to shopster container");
    let shopster_connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/stec_shopster_test", shopster_node.get_host_port_ipv4(5432).unwrap());

    test_code(tenet_connection_string, shopster_connection_string);

    shopster_node.stop().expect("Failed to stop shopster");
    tenet_node.stop().expect("Failed to stop tenet");
}
