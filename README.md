# weather-panel

build docker images from /weather
- docker build -f weather-server/Dockerfile -t weather-server .
- docker build -f weather-micro/Dockerfile -t weather-micro .

Or, `docker-compose up --build`