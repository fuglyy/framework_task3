<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Services\DashboardService;

/**
 * DashboardController: "Тонкий" контроллер.
 * Его задача - принять запрос, вызвать соответствующий Сервис и вернуть HTTP-ответ.
 * Вся бизнес-логика и агрегация перенесены в DashboardService.
 */
class DashboardController extends Controller
{
    private DashboardService $dashboardService;

    public function __construct(DashboardService $dashboardService)
    {
        // Dependency Injection: Laravel автоматически внедрит DashboardService
        $this->dashboardService = $dashboardService;
    }

    /**
     * Основная страница дашборда.
     */
    public function index()
    {
        // Вызываем Service Layer для получения всех необходимых данных
        $data = $this->dashboardService->getDashboardData();

        // Возвращаем данные в виде (ViewModel), готовом для использования в Blade.
        // Никаких HTTP-вызовов или логики агрегации здесь нет.
        return view('dashboard', $data);
    }

    /**
     * /api/jwst/feed — серверный прокси/нормализатор JWST картинок.
     * Логика фильтрации и нормализации данных перенесена в Сервис.
     */
    public function jwstFeed(Request $r)
    {
        // 1. Собираем параметры запроса
        $params = $r->only(['source', 'suffix', 'program', 'instrument', 'page', 'perPage']);
        
        // 2. Вызываем Service Layer для обработки запроса
        $result = $this->dashboardService->getJwstFeed($params);
        
        // 3. Возвращаем JSON-ответ
        return response()->json($result);
    }
    public function jwstPage()
    {
        return view('jwst');
    }

    public function getIssData(): array
    {
        return [
            'last'  => $this->issClient->getLast(),   // или как у тебя называется
            'trend' => $this->issClient->getTrend(),
            'base'  => '/api/iss',
        ];
    }

    public function issPage()
    {
        $data = $this->dashboardService->getIssData();

        return view('iss', $data);
    }




}