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
        $limit = (int) $request->query('limit', 50); // кол-во элементов на страницу
        $page  = max(1, (int) $request->query('page', 1));
        $search = $request->query('search', '');

        try {
            $allItems = $this->osdrService->getFlattenedOsdrList(); // без лимита, т.к. фильтруем сами

            // Фильтр поиска
            if ($search) {
                $allItems = array_filter($allItems, fn($row) =>
                    str_contains(strtolower($row['title'] ?? ''), strtolower($search)) ||
                    str_contains(strtolower($row['dataset_id'] ?? ''), strtolower($search))
                );
            }

            $total = count($allItems);
            $items = array_slice($allItems, ($page-1)*$limit, $limit);

            // Возвращаем JSON для AJAX
            if ($request->ajax()) {
                return response()->json([
                    'items' => array_values($items),
                    'page' => $page,
                    'totalPages' => ceil($total / $limit),
                    'total' => $total,
                    'perPage' => $limit,
                ]);
            }

            // Для обычного рендера страницы
            return view('osdr', [
                'items' => array_values($items),
                'page' => $page,
                'totalPages' => ceil($total / $limit),
                'limit' => $limit,
                'src' => (getenv('RUST_BASE') ?: 'http://rust_iss:3000') . '/osdr/list?limit=' . $limit,
            ]);

        } catch (\Exception $e) {
            throw $e;
        }
    }
}