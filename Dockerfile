FROM golang:1.20-alpine as builder
RUN apk update && apk add --no-cache git
WORKDIR /app
COPY backend/go.mod backend/go.sum ./
RUN go mod download
COPY backend/ .
RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o backend ./...

FROM alpine:latest
WORKDIR /app
COPY --from=builder /app/backend .
COPY frontend/out /app/frontend
ENV STATIC_FILES_DIR=/app/frontend
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
CMD ["/app/backend", "server"]
