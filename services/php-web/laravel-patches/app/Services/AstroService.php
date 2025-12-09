<?php

namespace App\Services;

use App\Clients\AstroApiClient;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Carbon;
use Exception;

/**
 * Service Layer для данных Астрономии (AstroController).
 * Содержит бизнес-логику расчёта дат, валидацию и кэширование.
 */
class AstroService
{
    private AstroApiClient $client;

    public function __construct(AstroApiClient $client)
    {
        // Dependency Injection: получаем AstroApiClient
        $this->client = $client;
    }

    /**
     * Получает список астрономических событий по координатам.
     * @param float $lat Широта
     * @param float $lon Долгота
     * @param int $days Количество дней для прогноза (1-30)
     * @return array
     * @throws Exception
     */
    public function getEvents(float $lat, float $lon, int $days): array
    {
        // 1. Бизнес-логика: Валидация и нормализация параметров
        $days = max(1, min(30, $days));
        
        // 2. Бизнес-логика: Расчет дат
        $from = Carbon::now('UTC')->toDateString();
        $to = Carbon::now('UTC')->addDays($days)->toDateString();
        
        $query = [
            'latitude' => $lat,
            'longitude' => $lon,
            'from' => $from,
            'to' => $to,
        ];
        
        // 3. Паттерн Кэширования: Запросы к внешним API дорогие и медленные.
        $cacheKey = 'astro_events_' . md5(json_encode($query));

        return Cache::remember($cacheKey, 7200, function () use ($query) {
            try {
                // 4. Вызов чистого клиента
                return $this->client->getEvents($query);
            } catch (Exception $e) {
                // В случае сбоя внешнего API, логируем и возвращаем унифицированный error-формат.
                // В зависимости от требования, можно вернуть заглушку или пробросить исключение выше.
                return [
                    'error' => $e->getMessage(), 
                    'code' => $e->getCode() ?: 500,
                    'is_cached_fallback' => false,
                ];
            }
        });
    }
}