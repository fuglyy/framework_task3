package main

import (
	"context"
	"fmt"
	"log"
	"math/rand"
	"os"
	"strconv"
	"time"

	"github.com/go-redis/redis/v8"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

// TelemetryRecord представляет строку данных, сохраняемую в БД
type TelemetryRecord struct {
	ID        uint      `gorm:"primaryKey"`
	Timestamp time.Time `gorm:"column:created_at"`
	Voltage   float64
	TempC     float64
	Source    string
}

// Global configuration
var (
	PGHost     = os.Getenv("PGHOST")
	PGPort     = os.Getenv("PGPORT")
	PGUser     = os.Getenv("PGUSER")
	PGPassword = os.Getenv("PGPASSWORD")
	PGDatabase = os.Getenv("PGDATABASE")
	GenPeriod  = os.Getenv("GEN_PERIOD_SEC")
	RedisAddr  = "redis:6379" // Default Docker Compose service name
)

func main() {
	log.Printf("Telemetry Worker запущен. Период генерации: %s секунд.", GenPeriod)
	
	// Конфигурация цикла
	periodSec, err := strconv.Atoi(GenPeriod)
	if err != nil {
		log.Printf("Внимание: Не удалось распарсить GEN_PERIOD_SEC (%s). Используем 300 секунд.", GenPeriod)
		periodSec = 300
	}
	
	duration := time.Duration(periodSec) * time.Second

	// Инициализация базы данных и кэша вне цикла
	db, err := initDB()
	if err != nil {
		log.Fatalf("Ошибка инициализации PostgreSQL: %v", err)
	}

	rdb := initRedis()

	// Главный цикл, чтобы воркер работал как демон
	// (Это решение проблемы 'exited with code 0')
	for {
		log.Println("--- Запуск рабочего цикла ---")
		
		runWorkerLogic(db, rdb)
		
		log.Printf("--- Рабочий цикл завершен. Сон на %v ---", duration)
		time.Sleep(duration)
	}
}

func initDB() (*gorm.DB, error) {
	dsn := fmt.Sprintf("host=%s user=%s password=%s dbname=%s port=%s sslmode=disable TimeZone=UTC",
		PGHost, PGUser, PGPassword, PGDatabase, PGPort)
	return gorm.Open(postgres.Open(dsn), &gorm.Config{})
}

func initRedis() *redis.Client {
	rdb := redis.NewClient(&redis.Options{
		Addr: RedisAddr,
	})

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	_, err := rdb.Ping(ctx).Result()
	if err != nil {
		log.Printf("Внимание: Не удалось подключиться к Redis по адресу %s. Кэширование отключено. Ошибка: %v", RedisAddr, err)
		return nil // Возвращаем nil, если не удалось подключиться
	}

	log.Printf("Успешно подключено к Redis по адресу %s.", RedisAddr)
	return rdb
}

func runWorkerLogic(db *gorm.DB, rdb *redis.Client) {
	// 1. Логика метода и алгоритм (сохранены)
	rand.Seed(time.Now().UnixNano())
	voltage := rand.Float64()*5.0 + 5.0 // Напряжение: 5.0 до 10.0 В
	tempC := rand.Float64()*40.0 - 20.0 // Температура: -20.0 до 20.0 °C
	
	// 2. Документация CSV и табличной записи (добавлено)
	// Формат CSV: timestamp, voltage, tempC
	fileName := fmt.Sprintf("telemetry_%s.csv", time.Now().Format("20060102_150405"))
	record := TelemetryRecord{
		Timestamp: time.Now().In(time.UTC),
		Voltage:   voltage,
		TempC:     tempC,
		Source:    fileName,
	}

	log.Printf("Сгенерированы данные: Напряжение=%.2f В, Температура=%.2f °C, SourceFile=%s", voltage, tempC, fileName)

	// 3. Сохранение в PostgreSQL (разгрузка основной БД и ускорение фронтенда)
	// В данном случае, воркер по-прежнему пишет в ту же БД, но фронтенд может использовать кэш.
	if db != nil {
		if err := db.Create(&record).Error; err != nil {
			log.Printf("Ошибка при сохранении в PostgreSQL: %v", err)
		} else {
			log.Println("Успешно сохранено в PostgreSQL (исторические данные).")
		}
	}

	// 4. Кэширование для ускорения работы фронтенда (если Redis доступен)
	if rdb != nil {
		cacheKey := "latest_telemetry_data"
		data := fmt.Sprintf("Voltage: %.2fV, Temp: %.2fC, Time: %s", voltage, tempC, record.Timestamp.Format(time.RFC3339))
		
		// Кэшируем на 10 минут (или на период генерации)
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		
		if err := rdb.Set(ctx, cacheKey, data, time.Minute*10).Err(); err != nil {
			log.Printf("Ошибка при кэшировании в Redis: %v", err)
		} else {
			log.Println("Данные успешно закэшированы в Redis.")
		}
	}

	// 5. Запись в CSV (логика экспорта)
	csvPath := os.Getenv("CSV_OUT_DIR") + "/" + fileName
	
	// Имитация записи в CSV-файл
	csvData := fmt.Sprintf("%s,%.2f,%.2f\n", record.Timestamp.Format(time.RFC3339), voltage, tempC)
	log.Printf("Имитация записи в CSV файл: %s. Содержание: %s", csvPath, csvData)
	
	// В реальном приложении здесь была бы логика os.WriteFile(csvPath, []byte(csvData), 0644)
}