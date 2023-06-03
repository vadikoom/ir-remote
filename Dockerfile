FROM golang:1.20-alpine as builder
RUN apk update && apk add --no-cache git
WORKDIR /app
COPY backend/go.mod backend/go.sum ./
RUN go mod download
COPY backend/ .
RUN CGO_ENABLED=0 GOOS=linux go build -o backend ./cmd/server

FROM alpine:latest
WORKDIR /app
COPY --from=builder /app/backend .
COPY frontend/out /app/frontend

ENV HTTP_PORT=8000
ENV HTTP_LISTEN_IP=0.0.0.0
ENV IR_LISTEN_IP=0.0.0.0
ENV IR_LISTEN_PORT=12000

EXPOSE 8000
CMD ["/app/backend", "server"]
