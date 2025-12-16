<?php

namespace App\Clients;

use Illuminate\Support\Facades\Http;
use Illuminate\Http\Client\PendingRequest;
use Exception;

/**
 * Клиент для взаимодействия с сервисом rust_iss.
 * Инкапсулирует базовый URL, таймауты и унифицированную обработку ошибок.
 *
 * Примечание: Для выполнения требования о едином формате ошибок (HTTP 200, ok=false)
 * мы настроим клиент так, чтобы он возвращал структурированный результат.
 */
class RustIssClient
{
    private PendingRequest $client;

    public function __construct()
    {
        $baseUrl = config('services.rust.base_url') ?? env('RUST_BASE', 'http://rust_iss:3000');

        $this->client = Http::baseUrl($baseUrl)
            ->timeout(5) // Таймаут 5 секунд
            ->retry(3, 100) // 3 ретрая с задержкой 100мс (Retry Pattern)
            ->withHeaders([
                'Accept' => 'application/json',
                'User-Agent' => 'Cassiopeia-Dashboard-Laravel/1.0',
            ]);
    }

    /**
     * Вызывает конечную точку и обрабатывает ответ.
     * @param string $path Путь API (например, '/last')
     * @param array $query Параметры запроса
     * @return array Результат в унифицированном формате (ok, payload/error)
     */
    private function getJson(string $path, array $query = []): array
    {
        try {
            $response = $this->client->get($path, $query);

            if ($response->successful() && $response->json('ok', true)) {
                // Успешный ответ (ok: true)
                return $response->json();
            }

            // Обработка случая, когда Rust-сервис возвращает ok: false с HTTP 200 (Требование ТЗ)
            if ($response->successful() && !$response->json('ok', true)) {
                return $response->json();
            }

            // Обработка HTTP ошибок (4xx, 5xx)
            return [
                'ok' => false,
                'error' => [
                    'code' => 'RUST_HTTP_' . $response->status(),
                    'message' => 'Rust service responded with status: ' . $response->status(),
                    'trace_id' => null, // Трассировку нужно добавить позже через Middleware
                ]
            ];

        } catch (Exception $e) {
            // Обработка сетевых ошибок (таймаут, DNS)
            return [
                'ok' => false,
                'error' => [
                    'code' => 'RUST_NETWORK_ERROR',
                    'message' => 'Network or Timeout error connecting to Rust service: ' . $e->getMessage(),
                    'trace_id' => null,
                ]
            ];
        }
    }

    /**
     * Получает последние данные о МКС.
     */
    public function getLast(): array
    {
        return $this->getJson('/last');
    }

    /**
     * Получает данные тренда МКС (фронт заберет сам, но оставляем заготовку).
     */
    public function getTrend(array $params = []): array
    {
        return $this->getJson('trend', $params);
    }
}