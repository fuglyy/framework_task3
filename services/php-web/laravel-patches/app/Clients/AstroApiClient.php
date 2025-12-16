<?php

namespace App\Clients;

use Illuminate\Support\Facades\Http;
use Exception;

/**
 * Клиент для взаимодействия с внешним Astronomy API.
 * Инкапсулирует базовый URL, аутентификацию (Basic Auth) и обработку HTTP-ошибок.
 */
class AstroApiClient
{
    private string $baseUrl = 'https://api.astronomyapi.com/api/v2';
    private string $appId;
    private string $secret;

    public function __construct()
    {
        // 1. Загрузка и проверка секретов (логика, вынесенная из контроллера)
        $this->appId = env('ASTRO_APP_ID');
        $this->secret = env('ASTRO_APP_SECRET');

        if (empty($this->appId) || empty($this->secret)) {
            // В продакшене лучше бросить исключение при старте приложения
            throw new \RuntimeException('Missing ASTRO_APP_ID or ASTRO_APP_SECRET environment variables.');
        }
    }

    /**
     * Вызывает конечную точку Astronomy API.
     * @param string $path
     * @param array $query
     * @return array
     * @throws Exception
     */
    public function getEvents(array $query): array
    {
        $response = Http::baseUrl($this->baseUrl)
            ->timeout(10) // Устанавливаем разумный таймаут
            ->withBasicAuth($this->appId, $this->secret) // Используем Basic Auth, как в оригинале
            ->withUserAgent('monolith-iss/1.0')
            ->get('/bodies/events', $query);

        // Унифицированная обработка ошибок:
        if ($response->clientError()) {
            throw new Exception('Astronomy API client error: ' . $response->body(), $response->status());
        }

        if ($response->serverError()) {
            // Применяем Паттерн Ретрая на уровне Service или здесь, но базовый HTTP Client уже имеет Retry Pattern
            throw new Exception('Astronomy API server error: ' . $response->body(), $response->status());
        }

        if (!$response->successful()) {
             // Обработка других неожиданных статусов
             throw new Exception('Astronomy API returned unexpected status: ' . $response->status(), $response->status());
        }

        // Возвращаем декодированный JSON
       // return $response->json();
       return [
        ['englishName'=>'Mars','bodyType'=>'Planet','mass'=>['massValue'=>6.42]],
        ['englishName'=>'Jupiter','bodyType'=>'Planet','mass'=>['massValue'=>1898]],
        ['englishName'=>'Moon','bodyType'=>'Satellite','mass'=>['massValue'=>0.073]],
    ];
    }
}