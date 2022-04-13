# Flock-rs
# Overview
Flock-rs is a scalable message broker inspired by a talk from LinkedIn that allows real-time events to be published over WebSocket connections.
# Components
Flock-rs consists of two components: flight and traffic-control, which need to be runned together in order for the component to work.
# Flight
Flight is the components that connects directly with end users over WebSocket connections. It allows user to connect to the component and subscribe or unsubscribe to a topic of interest. It can scales horizontally by running multiple instances adjacently. 
## Getting started
Before running the service, you must have a traffic-control and a watchtower service already running.

To start flight service, go inside flight folder and execute 
```
cargo run
```
By default, the the urls of the watchtower is `"http://localhost:8088"`. However, it can be set using the environment variable `WATCHTOWER_URLS`. On the other hand, the url of traffic-control will be acquired using the watchtower as a service discovery directly.
## Connecting with Flight
Unfortunately, there is currently no official client written for flight yet. However, you may implement custom client using the followings:
### Connection
To connect to a flight server simplies connect using WebSocket with the path of `/ws`
### Subscribe
To subscribe, send the payload with the following format:
```json
{
    "type": "Subscribe",
    "request_id": "[any_id]",
    "topic": "[your_topic]",
}
```
Then, you will receive the reply in the following format:
```json
{
    "type": "response",
    "topic": "[your_topic]",
    "subscribed": true,
    "request_id": "[your_id]"
}
```
### Unsubscribe
To unsubscribe, send the payload with the following format:
```json
{
    "type": "Unsubscribe",
    "request_id": "[any_id]",
    "topic": "[your_topic]",
}
```
Then, you will receive the reply in the following format:
```json
{
    "type": "response",
    "topic": "[your_topic]",
    "subscribed": false,
    "request_id": "[your_id]"
}
```

# Traffic-control
Traffic-control is a component that controls multiple flight instances. In order to publish events to end-user, you will need to publish to traffic-control.
## Getting Started
Traffic-control requires Redis for shared storage.You may run the redis using the `docker-compose.yml` file.
```
docker-compose up -d
```
Be sure to comment out other services that are not needed.

Then, to run traffic-control,
```
cargo run
```

## Connecting to traffic-control
### Rust Client
The library includes a Rust client. To include in your project, add the following to your Cargo.toml file.
```toml
traffic_control_client = { git = "https://github.com/warunyoud/flock-rs", branch = "main" }
```
The basic functionalities of the client can be described as followed:
```rust
use traffic_control_client::{TrafficControlClient, Error};

const USERNAME: &str = "admin";
const PASSWORD: &str = "password";

async fn main() {
    let traffic_control_client = TrafficControlClient::new(USERNAME, PASSWORD);

    // To publish
    let base_url = "http://127.0.0.1:8080";
    let topic = "mytopic";
    let payload = "{ \"message\": \"hello\" }";
    traffic_control_client.publish(base_url, topic, payload).await.unwrap();
}
```

### Python Client
To install the python client,
```
pip install traffic-control-client
```

Unlike the Rust client, in order to keep the service on the registry, you will have to manually call the ping function.
```python
from traffic_control_client import PyTrafficControlClient

traffic_control_client = PyTrafficControlClient("admin", "password")

# To publish
base_url = "http://127.0.0.1:8080"
topic = "mytopic"
payload = "{ \"message\": \"hello\" }"
traffic_control_client.publish(base_url, topic, payload) 
```
### Custom Client
You may write your own client and make the appropriate http requests in order to publish events.
# Limitations
Authentication for WebSocket client is not yet implemented. Please wait for newer version in the future.