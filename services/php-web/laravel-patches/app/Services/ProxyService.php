<?php

namespace App\Services;

use App\Clients\RustProxyClient;
use Exception;
use Illuminate\Support\Facades\Cache;

/**
 * Service Layer для проксирования запросов.
 * Изолирует контроллер от прямого взаимодействия с RustProxyClient.
 */
class ProxyService
{
    private RustProxyClient $client;

    public function __construct(RustProxyClient $client)
    {
        $this->client = $client;
    }

    /**
     * Проксирует запрос для получения последних данных.
     * @return array
     */
    public function getLastData(): array
    {
        // Можно добавить кэширование, если API не обновляется слишком часто.
        return $this->client->get('/last');
    }

    /**
     * Проксирует запрос для получения данных тренда.
     * @param array $qs Параметры запроса
     * @return array
     */
    public function getTrendData(array $qs = []): array
    {
        // Поскольку тренды могут быть специфичными и требовать свежих данных,
        // мы пока не добавляем кэширование.
        return $this->client->get('/iss/trend', $qs);
    }
}