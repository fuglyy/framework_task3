<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Services\AstroService;

class AstroController extends Controller
{
    private AstroService $astroService;

    public function __construct(AstroService $astroService)
    {
        $this->astroService = $astroService;
    }

    // Страница с таблицей
    public function index()
    {
        return view('astro');
    }

    // API для загрузки событий
    public function events(Request $r)
    {
        $lat = (float) $r->query('lat', 55.7558);
        $lon = (float) $r->query('lon', 37.6176);
        $days = (int) $r->query('days', 7);
        $limit = (int) $r->query('limit', 50); // лимит на фронт

        try {
            $data = $this->astroService->getEvents($lat, $lon, $days);
            if(isset($data['error'])) {
                return response()->json([
                    'ok' => false,
                    'error' => $data['error'],
                    'code' => $data['code'] ?? 500
                ], 200);
            }

            // Лимитируем количество элементов
            $data = array_slice($data, 0, $limit);

            return response()->json($data);
        } catch (\Exception $e) {
            return response()->json([
                'ok' => false,
                'error' => $e->getMessage()
            ], 500);
        }
    }
}
