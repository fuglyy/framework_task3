<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Services\AstroService; // Подключаем новый Service Layer

/**
 * AstroController: "Тонкий" контроллер.
 * Вся логика работы с внешним API и расчетом дат перенесена в AstroService.
 */
class AstroController extends Controller
{
    private AstroService $astroService;

    // Dependency Injection: Laravel автоматически внедрит AstroService
    public function __construct(AstroService $astroService)
    {
        $this->astroService = $astroService;
    }

    public function events(Request $r)
    {
        // 1. Получение и минимальная типизация параметров запроса.
        // Более сложная валидация должна быть в FormRequest, но здесь просто приведение типов.
        $lat = (float) $r->query('lat', 55.7558);
        $lon = (float) $r->query('lon', 37.6176);
        $days = (int) $r->query('days', 7);

        try {
            // 2. Вызов Service Layer.
            $result = $this->astroService->getEvents($lat, $lon, $days);

            // 3. Проверка на унифицированную ошибку (если сервис вернул 'error' в теле)
            if (isset($result['error'])) {
                 // Обработка требования "HTTP 200, ok: false"
                return response()->json([
                    'ok' => false, 
                    'error' => $result['error'],
                    'code' => $result['code'] ?? 500
                ], 200); // Возвращаем 200, но с ошибкой в теле
            }
            
            // 4. Успешный ответ
            return response()->json($result);

        } catch (\RuntimeException $e) {
            // Срабатывает, если не удалось загрузить секреты в AstroApiClient
            return response()->json([
                'ok' => false,
                'error' => 'Configuration error: ' . $e->getMessage()
            ], 500);
        } catch (\Exception $e) {
            // Catch-all для непредвиденных ошибок
            return response()->json([
                'ok' => false,
                'error' => 'Internal service error or unknown API failure.',
                'message' => $e->getMessage()
            ], 500);
        }
    }
}