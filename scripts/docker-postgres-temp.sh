docker run -d --rm -P -p 127.0.0.1:5432:5432 -e POSTGRES_PASSWORD="1234" --name pg postgres:alpine
# psql postgresql://postgres:1234@localhost:5432/postgres
