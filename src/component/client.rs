
use hyper::{Client as hClient, client::HttpConnector, Body as hBody};
use hyper_timeout::TimeoutConnector;
use hyper_tls::HttpsConnector;
use std::sync::Once;

pub type MClient = hClient<TimeoutConnector<HttpsConnector<HttpConnector>>>; 

pub struct Client;

impl Client {

    /// new static client 
    pub fn new() -> &'static Vec<MClient> {
        static INIT: Once = Once::new();
        static mut VAL: Vec<MClient> = Vec::new(); 
        unsafe{
            INIT.call_once(|| {
                    let https: HttpsConnector<HttpConnector> = HttpsConnector::new();
                    let mut conn = hyper_timeout::TimeoutConnector::new(https);
                    conn.set_connect_timeout(Some(std::time::Duration::from_secs(7)));
                    conn.set_read_timeout(Some(std::time::Duration::from_secs(23)));
                    conn.set_write_timeout(Some(std::time::Duration::from_secs(7)));
                    let clt = hClient::builder().build::<_, hBody>(conn); 
                    VAL.push( clt );
            });
            &VAL
        }
    }

}
