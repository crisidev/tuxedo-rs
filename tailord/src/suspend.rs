use std::{future::pending, time::Duration};

use futures_lite::StreamExt;
use tokio::sync::broadcast;
use zbus::{dbus_proxy, Connection};

#[dbus_proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait Suspend {
    #[dbus_proxy(signal)]
    fn prepare_for_sleep(&self, arg1: bool) -> fdo::Result<()>;
}

pub async fn wait_for_suspend(mut sender: broadcast::Sender<bool>) {
    // Don't try to reconnect anymore after 3 attempts
    for _ in 0..3 {
        tracing::info!("Setting up suspend service");
        if let Err(err) = try_wait_for_suspend(&mut sender).await {
            tracing::error!("Failed to wait for suspend: `{err}`");
            // Reconnect after 10s
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
    tracing::warn!("Stopping suspend service after 3 errors");
}

async fn try_wait_for_suspend(sender: &mut broadcast::Sender<bool>) -> Result<(), zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = SuspendProxy::new(&connection).await?;
    let mut receiver = proxy.receive_prepare_for_sleep().await?;

    while let Some(msg) = receiver.next().await {
        tracing::warn!("Received suspend message {msg:?}");
        let value = *msg.args()?.arg1();

        if value {
            tracing::info!("Suspended, sleeping until wake up.");
        } else {
            tracing::info!("Woken up, continue service.");
        };

        if let Err(err) = sender.send(value) {
            tracing::warn!("Error sending shutdown signal: `{err}`");
        }
    }

    Ok(())
}

pub async fn process_suspend(receiver: &mut broadcast::Receiver<bool>) {
    match receiver.recv().await {
        Ok(msg) => {
            // Suspended!
            if msg {
                tracing::warn!("We are suspended: {msg:?}");
                wait_for_wake_up(receiver).await
            } else {
                tracing::warn!("Wake up message without suspend.");
            }
        }
        Err(_) => {
            tracing::warn!("Stop listening for suspend messages");
            pending::<()>().await;
        }
    }
}

async fn wait_for_wake_up(receiver: &mut broadcast::Receiver<bool>) {
    // Wait until wake up (suspend msg == false).
    loop {
        match receiver.recv().await {
            Ok(msg) => {
                if msg {
                    tracing::warn!("Wake up message without suspend.");
                } else {
                    tracing::warn!("Nothing to do.");
                    return;
                }
            }
            Err(err) => {
                tracing::error!("Error receiving wake-up message: `{err}`");
            }
        }
    }
}
