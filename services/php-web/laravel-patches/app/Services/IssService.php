<?php

namespace App\Services;

use App\Clients\RustIssClient;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Log;

/**
 * Service Layer для данных МКС (ISS).
 * Ответственен за агрегацию, кэширование и нормализацию данных ISS.
 */
class IssService
{
    private RustIssClient $issClient;

    public function __construct(RustIssClient $issClient)
    {
        // Внедрение зависимости Rust API Client
        $this->issClient = $issClient;
    }

    /**
     * Получает актуальные данные и тренд для страницы ISS.
     * @return array
     */
    public function getIssDataForPage(): array
    {
        // Паттерн Кэширования: Снижаем нагрузку на rust_iss.
        // Кэшируем на 30 секунд.
        return Cache::remember('iss_page_data', 30, function () {
            // 1. Получение последних данных
            $lastData = $this->issClient->getIssLatest();
            $last = $lastData['payload'] ?? [];
            
            // 2. Получение данных тренда
            // Предполагаем, что /iss/trend не требует параметров для этой страницы
            $trendData = $this->issClient->getIssTrend();
            $trend = $trendData['payload'] ?? [];
            
            // Проверка на ошибку Rust-сервиса (если ок=false)
            if (!($lastData['ok'] ?? false) || !($trendData['ok'] ?? false)) {
                Log::error('Error fetching ISS data from Rust service', [
                    'last_error' => $lastData['error'] ?? 'N/A',
                    'trend_error' => $trendData['error'] ?? 'N/A',
                ]);
                
                // Возвращаем пустые данные или данные из другого кэша для устойчивости
                return $this->getFallbackData();
            }

            // Маппинг и трансформация (ViewModel)
            return [
                'last' => $last, 
                'trend' => $trend,
                // base URL может потребоваться для клиента, но в view его лучше не передавать
                // если только он не используется для JS-запросов напрямую
                // 'base' => $this->issClient->getBaseUrl(), 
            ];
        });
    }
    
    /**
     * Возвращает данные по умолчанию или из долгосрочного кэша при сбое.
     */
    private function getFallbackData(): array
    {
        return [
            'last' => Cache::get('iss_last_fallback', []),
            'trend' => Cache::get('iss_trend_fallback', []),
        ];
    }
}