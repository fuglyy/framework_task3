<?php

namespace App\Clients;

use Illuminate\Support\Facades\Http;
use Exception;

/**
 * Клиент для взаимодействия с Rust-сервисом OSDR (Open Space Data Repository).
 * Инкапсулирует базовый URL и логику HTTP-вызовов.
 */
class RustOsdrClient
{
    private string $baseUrl;

    public function __construct()
    {
        // Загрузка базового URL из ENV
        $this->baseUrl = getenv('RUST_BASE') ?: 'http://rust_iss:3000';

        // Проверка, что URL не пустой
        if (empty($this->baseUrl)) {
            throw new \RuntimeException('RUST_BASE environment variable is not configured.');
        }
    }

    /**
     * Получает список элементов OSDR.
     *
     * @param int $limit Ограничение на количество элементов.
     * @return array
     * @throws Exception
     */
    public function getOsdrList(int $limit = 20): array
    {
        $response = Http::baseUrl($this->baseUrl)
            ->timeout(10) // Разумный таймаут
            ->get('/osdr/list', ['limit' => $limit]);

        // Обработка ошибок
        if (!$response->successful()) {
            $status = $response->status();
            $body = $response->body();
            throw new Exception("OSDR Rust Service returned non-success status ({$status}): {$body}", $status);
        }

        // Возвращаем декодированный JSON
        $data = $response->json();
        
        // В случае пустого или невалидного JSON, возвращаем пустой массив.
        return is_array($data) ? $data : ['items' => []];
    }
}