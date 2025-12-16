<?php

namespace App\Services;

use App\Clients\AstroApiClient;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Carbon;
use Exception;

class AstroService
{
    private AstroApiClient $client;

    public function __construct(AstroApiClient $client)
    {
        $this->client = $client;
    }

    /**
     * Получение астрономических событий
     */
    public function getEvents(float $lat, float $lon, int $days): array
    {
        $days = max(1, min(30, $days));

        $from = Carbon::now('UTC')->toDateString();
        $to = Carbon::now('UTC')->addDays($days)->toDateString();

        $query = [
            'latitude' => $lat,
            'longitude' => $lon,
            'from' => $from,
            'to' => $to,
        ];

        $cacheKey = 'astro_events_' . md5(json_encode($query));

        return Cache::remember($cacheKey, 7200, function () use ($query) {
            try {
                return $this->client->getEvents($query);
            } catch (Exception $e) {
                return [
                    'error' => $e->getMessage(),
                    'code' => $e->getCode() ?: 500,
                ];
            }
        });
    }
}
