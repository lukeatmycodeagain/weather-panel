
# Environment Setup

### Important Variables:
- **`ROCKET_PORT`**: The port for the Rocket web server. Defaults to 8000
- **`WEATHER_MICROSERVICE_PORT`**: The port for the weather microservice.  Defaults to 8080
- **`OPEN_WEATHER_API`**: Required API key from [OpenWeather](https://openweathermap.org/).  

### Set the Weather Microservice URL when running outside a container:
```env
WEATHER_MICROSERVICE_URL=http://localhost
```

## Building and running with `cargo`
```env
cd ~/weather/weather-utils && cargo build
cd ~/weather/weather-micro && cargo run
cd ~/weather/weather-server && cargo run
```

## Containers
### Build Docker Images:
Run the following commands from the root directory:  
```bash
docker build -f weather-server/Dockerfile -t weather-server .
docker build -f weather-micro/Dockerfile -t weather-micro .
```

Alternatively, use Docker Compose to build and run the services:  
```bash
docker-compose up --build
```