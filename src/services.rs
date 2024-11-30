use deppy::{Initialize, ServiceCollectionBuilder, ServiceHandler};
use twilight_cache_inmemory::InMemoryCache;

#[derive(Clone)]
struct InitializeHttp {
    token: String,
}

impl Initialize<twilight_http::Client> for InitializeHttp {
    fn initialize<T: ServiceHandler>(&self, _: &T) -> twilight_http::Client {
        twilight_http::Client::new(self.token.clone())
    }
}

#[derive(Clone)]
struct InitializeTwilight;

impl Initialize<InMemoryCache> for InitializeTwilight {
    fn initialize<T: ServiceHandler>(&self, _: &T) -> InMemoryCache {
        InMemoryCache::new()
    }
}

pub trait AddTwilightServices {
    fn add_http_client(self, token: String) -> Self;

    fn add_in_memory_cache(self) -> Self;
}

impl AddTwilightServices for ServiceCollectionBuilder {
    fn add_http_client(self, token: String) -> Self {
        self.add_service(deppy::ServiceType::Singleton, InitializeHttp { token })
    }

    fn add_in_memory_cache(self) -> Self {
        self.add_service(deppy::ServiceType::Singleton, InitializeTwilight)
    }
}
