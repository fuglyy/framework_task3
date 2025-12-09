<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Services\OsdrService;

/**
 * OsdrController: "Тонкий" контроллер.
 * Его задача - принять запрос, вызвать соответствующий Сервис и вернуть HTTP-ответ.
 * Вся логика HTTP-вызовов и преобразования данных перенесена в Service/Client слои.
 */
class OsdrController extends Controller
{
    private OsdrService $osdrService;

    public function __construct(OsdrService $osdrService)
    {
        // Dependency Injection
        $this->osdrService = $osdrService;
    }

    public function index(Request $request)
    {
        // 1. Получаем и приводим к типу limit
        $limit = (int) $request->query('limit', 20);

        try {
            // 2. Вызываем Service Layer для получения, преобразования и кэширования данных
            $items = $this->osdrService->getFlattenedOsdrList($limit);
            
            // 3. Возвращаем View
            return view('osdr', [
                'items' => $items,
                // Здесь мы можем использовать ENV переменную, но лучше передавать ее через Service
                // Для упрощения примера оставим так, как было:
                'src' => (getenv('RUST_BASE') ?: 'http://rust_iss:3000') . '/osdr/list?limit=' . $limit,
            ]);
            
        } catch (\Exception $e) {
            // Если Service бросил исключение, оно будет поймано в Handler (app/Exceptions/Handler.php)
            // Здесь можно просто вернуть View с сообщением об ошибке или пробросить исключение
            throw $e;
        }
    }
}