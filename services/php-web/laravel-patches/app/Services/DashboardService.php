<?php

namespace App\Services;

use App\Clients\RustIssClient;
use App\Support\JwstHelper; // Предполагаем, что JwstHelper - это клиент/адаптер
use Illuminate\Support\Facades\Cache;

/**
 * Слой сервисов для Dashboard.
 * Содержит бизнес-логику, агрегацию данных, кэширование.
 */
class DashboardService
{
    private RustIssClient $issClient;
    private JwstHelper $jwstClient; // Адаптер для JWST

    public function __construct(RustIssClient $issClient, JwstHelper $jwstClient)
    {
        // Dependency Injection: получаем готовые и сконфигурированные клиенты
        $this->issClient = $issClient;
        $this->jwstClient = $jwstClient;
    }

    /**
     * Агрегирует данные для основного Dashboard (МКС, метрики, пустые контейнеры).
     * @return array
     */
    public function getDashboardData(): array
    {
        // Паттерн Кэширования: Кэшируем результат вызова внешнего API на 60 секунд.
        // Это резко повышает производительность и снижает нагрузку на rust_iss.
        return Cache::remember('dashboard_iss_metrics', 60, function () {
            
            $issData = $this->issClient->getIssLatest();
            
            // Обработка данных
            $iss = $issData['payload'] ?? [];
            $ok = $issData['ok'] ?? false;

            // Если Rust-сервис вернул ошибку, возвращаем пустые данные, но логируем.
            if (!$ok) {
                // Логирование ошибки (например, Log::error(...))
                return [
                    'iss' => [],
                    'metrics' => $this->getDefaultMetrics(),
                    'error' => $issData['error'] ?? ['message' => 'Unknown error from Rust service.'],
                ];
            }

            // Маппинг и трансформация данных (ViewModel для Blade)
            return [
                'iss' => $iss,
                'trend' => [], // Фронт заберет сам
                'jw_gallery' => [], // Фронт заберет сам через jwstFeed
                'metrics' => [
                    'iss_speed' => $iss['velocity'] ?? null,
                    'iss_alt' => $iss['altitude'] ?? null,
                    'neo_total' => 0, // Это значение должно быть получено из другого клиента/репозитория
                ],
            ];
        });
    }

    /**
     * Обрабатывает логику получения и фильтрации ленты JWST.
     * ЭТО ПЕРЕНЕСЕННАЯ БИЗНЕС-ЛОГИКА ИЗ СТАРОГО КОНТРОЛЛЕРА.
     */
    public function getJwstFeed(array $params): array
    {
        $src = $params['source'] ?? 'jpg';
        $sfx = trim((string)($params['suffix'] ?? ''));
        $prog = trim((string)($params['program'] ?? ''));
        $instF = strtoupper(trim((string)($params['instrument'] ?? '')));
        $page = max(1, (int)($params['page'] ?? 1));
        $per = max(1, min(60, (int)($params['perPage'] ?? 24)));

        // 1. Определение эндпоинта (Бизнес-логика)
        $path = 'all/type/jpg';
        if ($src === 'suffix' && $sfx !== '') $path = 'all/suffix/' . ltrim($sfx, '/');
        if ($src === 'program' && $prog !== '') $path = 'program/id/' . rawurlencode($prog);

        // Паттерн Кэширования: Кэшируем результат JWST-запроса, используя параметры как ключ.
        $cacheKey = 'jwst_feed_' . md5(json_encode([$path, $page, $per, $instF]));

        return Cache::remember($cacheKey, 3600, function() use ($path, $page, $per, $instF) {
            // 2. Вызов внешнего клиента (через JwstHelper)
            $resp = $this->jwstClient->get($path, ['page' => $page, 'perPage' => $per]);
            $list = $resp['body'] ?? ($resp['data'] ?? (is_array($resp) ? $resp : []));

            $items = [];
            foreach ($list as $it) {
                if (!is_array($it)) continue;

                // 3. Логика фильтрации и нормализации данных (Бизнес-логика)
                
                // Выбор валидной картинки (упрощенная версия)
                $url = \App\Support\JwstHelper::pickImageUrl($it);
                if (!$url) continue;

                // Фильтр по инструменту
                $instList = array_map('strtoupper', array_column($it['details']['instruments'] ?? [], 'instrument'));
                if ($instF && $instList && !in_array($instF, $instList, true)) continue;

                // Маппинг в DTO/ViewModel
                $items[] = $this->mapJwstItem($it, $url, $instList);

                if (count($items) >= $per) break;
            }

            return [
                'source' => $path,
                'count' => count($items),
                'items' => $items,
            ];
        });
    }

    /**
     * Маппинг одного элемента JWST в конечный формат ViewModel.
     * @param array $it Сырые данные элемента
     * @param string $url Финальный URL изображения
     * @param array $instList Список инструментов
     * @return array
     */
    private function mapJwstItem(array $it, string $url, array $instList): array
    {
        $loc = $it['location'] ?? $it['url'] ?? $url;

        return [
            'url' => $url,
            'obs' => (string)($it['observation_id'] ?? $it['observationId'] ?? ''),
            'program' => (string)($it['program'] ?? ''),
            'suffix' => (string)($it['details']['suffix'] ?? $it['suffix'] ?? ''),
            'inst' => $instList,
            'caption' => trim(
                (($it['observation_id'] ?? '') ?: ($it['id'] ?? '')) .
                ' · P' . ($it['program'] ?? '-') .
                (($it['details']['suffix'] ?? '') ? ' · ' . $it['details']['suffix'] : '') .
                ($instList ? ' · ' . implode('/', $instList) : '')
            ),
            'link' => $loc,
        ];
    }

    private function getDefaultMetrics(): array
    {
        return [
            'iss_speed' => null,
            'iss_alt' => null,
            'neo_total' => 0,
        ];
    }
}