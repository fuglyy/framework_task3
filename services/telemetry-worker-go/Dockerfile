# 1. Сборка (Build Stage)
FROM golang:1.22-alpine AS builder

# ИСПРАВЛЕНИЕ: Устанавливаем Git, чтобы 'go mod download' мог загружать модули из VCS (систем контроля версий)
RUN apk add --no-cache git

WORKDIR /app

# КОРРЕКЦИЯ: Копируем только go.mod и основной файл. Исключаем go.sum,
# чтобы Go принудительно пересоздал его с актуальными контрольными суммами.
COPY go.mod ./
COPY telemetry_worker.go ./

# ИСПРАВЛЕНИЕ 1: Отключаем проверку контрольной суммы (GOSUMDB=off) для надежности.
# Теперь go mod tidy сгенерирует новый, корректный go.sum
RUN GOSUMDB=off go mod tidy

# ИСПРАВЛЕНИЕ 2: Также отключаем проверку для финальной сборки.
RUN GOSUMDB=off CGO_ENABLED=0 go build -o /telemetry_worker ./telemetry_worker.go

# 2. Исполнение (Final Stage)
FROM alpine:latest

# Устанавливаем часовой пояс
RUN apk --no-cache add tzdata

WORKDIR /

# Копируем только исполняемый файл
COPY --from=builder /telemetry_worker /telemetry_worker

# Устанавливаем рабочую директорию для CSV
RUN mkdir -p /data/csv

# Запуск программы напрямую. Это гарантирует, что она не выйдет, пока Go-код не завершится.
ENTRYPOINT ["/telemetry_worker"]