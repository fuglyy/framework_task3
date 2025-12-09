<?php

namespace App\Clients;

use Illuminate\Support\Facades\Http;
use Exception;
use Illuminate\Http\Client\PendingRequest;

/**
 * Клиент для взаимодействия с JWST API (ранее JwstHelper).
 * Использует Laravel Http Client вместо @file_get_contents.
 * Инкапсулирует базовый URL, заголовки (x-api-key, email) и обработку ошибок.
 */
class JwstApiClient
{
    private PendingRequest $client;
    private string $host;

    public function __construct()
    {
        // Логика получения настроек
        $this->host = rtrim(getenv('JWST_HOST') ?: 'https://api.jwstapi.com', '/');
        $key = getenv('JWST_API_KEY') ?: '';
        $email = getenv('JWST_EMAIL') ?: null;

        if (empty($key)) {
            // Критическая ошибка конфигурации
            throw new \RuntimeException('JWST_API_KEY is not configured.');
        }

        // 1. Инициализация Http Client
        $this->client = Http::baseUrl($this->host)
            ->timeout(12) // Таймаут из оригинального кода
            ->retry(2, 500) // 2 ретрая с задержкой 500мс
            ->withHeaders([
                'Accept' => 'application/json',
                'User-Agent' => 'Cassiopeia-Dashboard-Laravel/1.0',
                'x-api-key' => $key,
            ]);

        // Добавление Email, если он есть
        if ($email) {
             $this->client->withHeaders(['email' => $email]);
        }
    }

    /**
     * Выполняет GET-запрос к JWST API.
     * @param string $path Путь API (например, 'all/type/jpg')
     * @param array $qs Параметры запроса
     * @return array Результат в виде ассоциативного массива.
     * @throws Exception
     */
    public function get(string $path, array $qs = []): array
    {
        try {
            $response = $this->client->get('/' . ltrim($path, '/'), $qs);

            if ($response->clientError()) {
                throw new Exception('JWST API client error: ' . $response->body(), $response->status());
            }

            if ($response->serverError()) {
                throw new Exception('JWST API server error: ' . $response->body(), $response->status());
            }

            if (!$response->successful()) {
                 // Обработка других неожиданных статусов
                 throw new Exception('JWST API returned unexpected status: ' . $response->status(), $response->status());
            }

            // Возвращаем декодированный JSON
            return $response->json() ?? [];
        } catch (Exception $e) {
            // В случае сетевой ошибки или таймаута
            throw new Exception('Network or API Error while connecting to JWST API: ' . $e->getMessage(), $e->getCode() ?: 500);
        }
    }

    /**
     * Статическая функция для выбора лучшего URL изображения из сырого элемента.
     * (Оставляем ее в Клиенте/Адаптере, так как она специфична для структуры данных JWST).
     */
    public static function pickImageUrl(array $item): ?string
    {
        $keys = ['thumbnail','thumbnailUrl','image','img','url','href','link','s3_url','file_url'];
        foreach ($keys as $k) {
            $v = $item[$k] ?? null;
            if (is_string($v)) {
                $u = trim($v);
                if (preg_match('~^https?://~i',$u) && preg_match('~\.(jpg|jpeg|png)$~i',$u)) return $u;
                if (str_starts_with($u,'/') && preg_match('~\.(jpg|jpeg|png)$~i',$u)) return 'https://api.jwstapi.com'.$u;
            }
        }
        foreach ($item as $v) if (is_array($v)) { $u = self::pickImageUrl($v); if ($u) return $u; }
        return null;
    }
}