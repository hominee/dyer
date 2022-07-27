use crate::client::{Client, ClientType, CLIENTPOOL};
use http::header::{HeaderName, HeaderValue};
use hyper::client::HttpConnector;
use hyper_proxy::{Intercept, Proxy as hProxy, ProxyConnector};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

/// data type that represent proxy
#[derive(Clone, Serialize, Default, fmt::Debug, Deserialize)]
pub struct Proxy {
    pub(crate) addr: String,
    pub(crate) auth: Option<Auth>,
}

impl Proxy {
    pub fn new<T: Into<String>>(addr: T) -> Self {
        Self {
            addr: addr.into(),
            auth: None,
        }
    }

    pub fn set_addr<T: Into<String>>(&mut self, addr: T) {
        self.addr = addr.into();
    }

    pub fn set_auth_bearer(&mut self, bearer: String) {
        let bearerauth = AuthBearer { bearer };
        self.auth = Some(Auth::Bearer(bearerauth));
    }

    pub fn set_auth_basic(&mut self, username: String, password: String) {
        let basicauth = AuthBasic { username, password };
        self.auth = Some(Auth::Basic(basicauth));
    }

    pub fn set_auth_custom(&mut self, token: String) {
        let customauth = AuthCustom { token };
        self.auth = Some(Auth::Custom(customauth));
    }

    pub fn addr(&self) -> &str {
        self.addr.as_ref()
    }

    pub fn auth(&self) -> Option<&Auth> {
        self.auth.as_ref()
    }

    pub fn addr_mut(&mut self) -> &mut str {
        self.addr.as_mut()
    }

    pub fn auth_mut(&mut self) -> Option<&mut Auth> {
        self.auth.as_mut()
    }
}

impl Proxy {
    pub fn build(&self) -> &'static Client {
        let id = crate::utils::hash(Some(&self));
        let uri = self.addr.parse().unwrap();
        let mut prx = hProxy::new(Intercept::All, uri);
        if let Some(auth) = &self.auth {
            let name = HeaderName::from_str("Authorization").unwrap();
            let val = HeaderValue::from_str(&auth.encode()).unwrap();
            prx.set_header(name, val);
        }
        let conn = HttpConnector::new();
        let prxconn = ProxyConnector::from_proxy(conn, prx).unwrap();
        let client = hyper::Client::builder().build::<_, hyper::Body>(prxconn);
        let downloader = Client {
            id,
            inner: ClientType::Proxy(client),
        };
        unsafe {
            match CLIENTPOOL {
                Some(ref mut pool) => {
                    pool.insert(id, downloader);
                }
                None => {
                    let mut pool = HashMap::new();
                    pool.insert(id, downloader);
                    CLIENTPOOL = Some(pool);
                }
            }
            CLIENTPOOL.as_ref().unwrap().get(&id).unwrap()
        }
    }
}

impl Hash for Proxy {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.addr.hash(state);
    }
}

/// proxy authentication
/// it support 3 types:
/// 1) Basic: username & password, the format is `Authorization: Basic username:password`
/// 2) Bearer: Token, the format is `Authorization: Bearer token`
/// 3) Custom: custom Token, the format is `Authorization: token`, Note that be care for the format
///    of customized token string
#[derive(Clone, Serialize, fmt::Debug, Deserialize)]
#[serde(untagged)]
pub enum Auth {
    Basic(AuthBasic),
    Bearer(AuthBearer),
    Custom(AuthCustom),
}

impl Auth {
    pub fn encode(&self) -> String {
        match self {
            Auth::Basic(au) => {
                let b64 = base64::encode(format!("{}:{}", au.username, au.password));
                format!("Basic {}", b64)
            }
            Auth::Bearer(au) => {
                let b64 = base64::encode(format!("{}", au.bearer));
                format!("Bearer {}", b64)
            }
            Auth::Custom(au) => au.token.clone(),
        }
    }
}

#[derive(Clone, Serialize, fmt::Debug, Deserialize)]
pub struct AuthBasic {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Clone, Serialize, fmt::Debug, Deserialize)]
pub struct AuthBearer {
    pub(crate) bearer: String,
}

#[derive(Clone, Serialize, fmt::Debug, Deserialize)]
pub struct AuthCustom {
    pub(crate) token: String,
}
