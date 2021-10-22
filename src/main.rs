use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use std::io;

async fn spawn_tcpread<A>(addr: A) -> io::Result<JoinHandle<()>> 
where A: ToSocketAddrs {
    let listener = TcpListener::bind(addr).await?;
    // tokio::spawn()

    let handle = tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.unwrap();
        }
    });

    Ok(handle)
}

async fn spawn_tcpread_select<A>(addr: A, token: CancellationToken) -> io::Result<JoinHandle<()>> 
where A: ToSocketAddrs {
    let listener = TcpListener::bind(addr).await?;
    // tokio::spawn()

    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                result = listener.accept() => {
                    let (socket, _) = result.unwrap();
                },
                _ = token.cancelled() => {
                    return;
                }
            }
        }
    });

    Ok(handle)
}




#[tokio::main]
async fn main() -> io::Result<()> {

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::{Abortable, AbortHandle};
    use tokio::sync::mpsc;
    use std::time::Duration;
    use tokio::time::sleep;
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn test_spawn_tcpread_success() -> io::Result<()> {
        let _handle = spawn_tcpread("127.0.0.1:8080").await?;

        assert!(TcpStream::connect("127.0.0.1:8080").await.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_spawn_tcpread_shutdown() -> io::Result<()> {
        let handle = spawn_tcpread("127.0.0.1:8081").await?;
        handle.abort();

        println!("{:?}", TcpStream::connect("127.0.0.1:8081").await);
        assert!(TcpStream::connect("127.0.0.1:8081").await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_spawn_tcpread_shutdown_future() -> io::Result<()> {
        let handle = spawn_tcpread("127.0.0.1:8084").await?;

        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let future = Abortable::new(handle, abort_registration);

        tokio::spawn(async move {
            let _ = future.await;
        });

        abort_handle.abort();

        for _ in 0..100 {
            tokio::spawn(async move {1});
            sleep(Duration::from_millis(10)).await;
        }

        assert!(TcpStream::connect("127.0.0.1:8084").await.is_err());
        Ok(())
    }

    // #[tokio::test]
    // async fn test_spawn_tcpread_shutdown_select() -> io::Result<()> {
    //     let (tx, mut rx) = mpsc::unbounded_channel();
    //     let handle = spawn_tcpread("127.0.0.1:8084").await?;

    //     tokio::spawn(async move {
    //         tokio::select! {
    //             _ = rx.recv() => {
    //                 handle.abort(); // try to borrow moved value
    //             }
    //             v = handle => {},
    //         }
    //     });

    //     tx.send(()).unwrap();

    //     assert!(TcpStream::connect("127.0.0.1:8084").await.is_err());
    //     Ok(())
    // }

    #[tokio::test]
    async fn test_spawn_tcpread_shutdown_token() -> io::Result<()> {
        let token = CancellationToken::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let handle = spawn_tcpread_select("127.0.0.1:8085", token.clone()).await?;

        tokio::spawn(async move {
            tokio::select! {
                _ = rx.recv() => {
                    token.cancel();
                }
                v = handle => {},
            }
        });

        tx.send(()).unwrap();

        assert!(TcpStream::connect("127.0.0.1:8085").await.is_err());
        Ok(())
    }
}
