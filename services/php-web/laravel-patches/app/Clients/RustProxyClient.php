<?php

namespace App\Clients;

use Illuminate\Support\Facades\Http;
use Exception;
use Illuminate\Http\Client\PendingRequest;

/**
 * Клиент для проксирования запросов к Rust-сервису (RUST_BASE).
 * Использует Laravel Http Client, обеспечивает таймаут и обработку ошибок.
 */
class RustProxyClient
{
    private PendingRequest $client;
    private string $baseUrl;

    public function __construct()
    {
        $this->baseUrl = rtrim(getenv('RUST_BASE') ?: 'http://rust_iss:3000', '/');
        
        $this->client = Http::baseUrl($this->baseUrl)
            ->timeout(5) // Таймаут из оригинального кода (5 секунд)
            ->acceptJson(); // Принимаем JSON-ответ
    }

    /**
     * Выполняет проксирование GET-запроса к Rust-сервису.
     *
     * @param string $path Путь API (например, '/last', '/iss/trend')
     * @param array $qs Параметры запроса (Query String)
     * @return array Результат в виде ассоциативного массива.
     * @throws Exception
     */
    public function get(string $path, array $qs = []): array
    {
        try {
            $response = $this->client->get($path, $qs);

            // Обработка ошибок. Поскольку это прокси, мы ожидаем успешный статус.
            if ($response->failed()) {
                $status = $response->status();
                $body = $response->body();
                throw new Exception("Rust Proxy request failed with status ({$status}): {$body}", $status);
            }

            // Возвращаем декодированный JSON (или пустой массив в случае пустого ответа)
            return $response->json() ?? [];

        } catch (\Exception $e) {
            // Перехватываем сетевые ошибки (Connection Timeout, DNS failures)
            throw new Exception("Proxy failed to connect to {$this->baseUrl}: " . $e->getMessage(), $e->getCode() ?: 500);
        }
    }
    
    public function getBaseUrl(): string
    {
        return $this->baseUrl;
    }
}