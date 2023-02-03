pub mod prelude{

    use tokio::{io::{self, AsyncReadExt},net::{TcpListener,TcpStream}, join,sync::Mutex};
    use tokio::macros::support::Future;
    use std::{fmt,error::Error};
    use std::{collections::HashMap, sync::{Arc}};
    use std::sync::Once;
    use core::pin::Pin;

    // ServErrorBuild is server::Build, an application instance can only be assembled once
    #[derive(Debug)]
    pub struct ServErrorBuild;

    impl fmt::Display for ServErrorBuild {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "ServErrorBuild is server::Build, an application instance can only be assembled once")
        }
    }

    static mut STATE_APP: usize = 0;
    static INIT: Once = Once::new();

    // update Handler later
    type Handler = fn(TcpStream) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

    pub trait servConfig {
        //return String it is temporarily, next version return handler
        fn getRoad(&self, addr: String) -> &Handler;
        fn setRoad(&mut self, addr: String, handler: Handler);
    }

    trait App: servConfig{
        fn new() -> Box<dyn servConfig>;
    }

    struct ServerRoads{
        roads: Roads,
    }
    struct Roads{
        mapRoads: HashMap<String, Handler>,
    }
    struct Conection{
        tcpListener: TcpListener,
    }

    pub struct Application{
        conection: Conection,
        server_roads: ServerRoads,
    }

    struct MementoApplication{
        historyApp: Vec<Application>,
    }

    impl servConfig for Application {
        fn getRoad(&self, addr: String) -> &Handler{
            //example
            self.server_roads.roads.mapRoads.get(&addr).unwrap()
        }

        fn setRoad(&mut self, addr: String, handler: Handler){
            //example 
            //create ERROR later
            self.server_roads.roads.mapRoads.insert(addr, handler);
            
        }
    }

    impl Application {

    async fn new(addrListener: String) -> Application{
            //Create new object Application
            Application{
                //create error later
                conection: Conection { tcpListener: TcpListener::bind(addrListener).await.unwrap() },
                server_roads: ServerRoads {
                    roads: Roads {
                        mapRoads: HashMap::new()
                } }
            }
        } 

        pub async fn Build(addrListener: String) -> Result<Option<Application>, ServErrorBuild>{
            unsafe{
                if STATE_APP == 1 {
                    return Err(ServErrorBuild);
                }
                let app = Application::new(addrListener).await;
                INIT.call_once(|| {
                    STATE_APP = 1;
                });
                Ok(Some(app))
            }
        }

        pub async fn run(self: Arc<Self>) -> Result<(), io::Error>{
            loop {
                //handler err
                let copy_app = self.clone();
                let mut stream = copy_app.conection.tcpListener.accept().await;
                if let Err(err) = stream{
                    return Err(err);
                }else if let Ok( mut stream) = stream {   
                    let handler_current = Self::Get_handler_to_request(copy_app, Arc::new(Mutex::new(&mut stream.0)))
                                    .await;
                                    
                    if let Some(handler) =  handler_current {
                            tokio::spawn(async move{    
                                    processing_request(stream.0, handler).await;
                            });
                    }
                    
                }   
            }
        }
        
        async fn Get_handler_to_request(self: Arc<Self>, stream: Arc<tokio::sync::Mutex<&mut tokio::net::TcpStream>>) -> Option<Handler>{
            let mut buffer = vec![0;1024];
            stream.lock().await.read(&mut buffer).await.unwrap();
            let keys = self.server_roads.roads.mapRoads.keys();
            for key in keys {
                if buffer.starts_with(key.as_bytes()){
                    return Some(self.getRoad(key.into()).clone());
                }
            }
            None
        }

    }

    async fn processing_request(stream: TcpStream, handler: Handler){
        //processing of an incoming request with a user-defined function
        handler(stream).await;
    }
}
