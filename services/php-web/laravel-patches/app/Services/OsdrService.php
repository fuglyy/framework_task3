<?php

namespace App\Services;

use App\Clients\RustOsdrClient;
use Illuminate\Support\Facades\Cache;
use Exception;

/**
 * Service Layer для работы с данными OSDR.
 * Содержит бизнес-логику преобразования, нормализации и кэширования.
 */
class OsdrService
{
    private RustOsdrClient $client;

    public function __construct(RustOsdrClient $client)
    {
        // Dependency Injection
        $this->client = $client;
    }

    /**
     * Получает и преобразует данные OSDR для вывода в шаблоне.
     * Применяется кэширование, так как это внешний вызов.
     *
     * @param int $limit
     * @return array
     * @throws Exception
     */
    public function getFlattenedOsdrList(int $limit = 20): array
    {
        $limit = max(1, min(100, $limit)); // Безопасное ограничение

        $cacheKey = 'osdr_list_limit_' . $limit;

        // Кэширование на 10 минут
        return Cache::remember($cacheKey, 600, function () use ($limit) {
            try {
                // 1. Вызов внешнего клиента
                $data = $this->client->getOsdrList($limit);
                $items = $data['items'] ?? [];

                // 2. Применение бизнес-логики преобразования
                $flattenedItems = $this->flattenOsdr($items);
                
                return $flattenedItems;

            } catch (Exception $e) {
                // В случае сбоя клиента, логируем и пробрасываем исключение
                // Оно будет поймано в центральном Handler (app/Exceptions/Handler.php)
                throw new Exception("Failed to retrieve OSDR data: " . $e->getMessage(), $e->getCode() ?: 500);
            }
        });
    }

    /**
     * Преобразует данные вида {"OSD-1": {...}, "OSD-2": {...}} в плоский список (ключевая логика).
     * @param array $items
     * @return array
     */
    private function flattenOsdr(array $items): array
    {
        $out = [];
        foreach ($items as $row) {
            $raw = $row['raw'] ?? [];
            if (is_array($raw) && $this->looksOsdrDict($raw)) {
                foreach ($raw as $k => $v) {
                    if (!is_array($v)) continue;
                    $rest = $v['REST_URL'] ?? $v['rest_url'] ?? $v['rest'] ?? null;
                    $title = $v['title'] ?? $v['name'] ?? null;
                    if (!$title && is_string($rest)) {
                        // запасной вариант: последний сегмент URL как подпись
                        $title = basename(rtrim($rest, '/'));
                    }
                    $out[] = [
                        'id'          => $row['id'] ?? null,
                        'dataset_id'  => $k,
                        'title'       => $title,
                        'status'      => $row['status'] ?? null,
                        'updated_at'  => $row['updated_at'] ?? null,
                        'inserted_at' => $row['inserted_at'] ?? null,
                        'rest_url'    => $rest,
                        'raw'         => $v,
                    ];
                }
            } else {
                // обычная строка — просто прокинем REST_URL если найдётся
                $row['rest_url'] = is_array($raw) ? ($raw['REST_URL'] ?? $raw['rest_url'] ?? null) : null;
                $out[] = $row;
            }
        }
        return $out;
    }

    /**
     * Проверяет, похоже ли содержимое на словарь OSDR.
     * @param array $raw
     * @return bool
     */
    private function looksOsdrDict(array $raw): bool
    {
        // словарь ключей "OSD-xxx" ИЛИ значения содержат REST_URL
        foreach ($raw as $k => $v) {
            if (is_string($k) && str_starts_with($k, 'OSD-')) return true;
            if (is_array($v) && (isset($v['REST_URL']) || isset($v['rest_url']))) return true;
        }
        return false;
    }
}