<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Services\IssService; // Подключаем новый Service Layer

/**
 * IssController: "Тонкий" контроллер.
 * Вся логика работы с Rust API перенесена в IssService.
 */
class IssController extends Controller
{
    private IssService $issService;

    // Dependency Injection: Laravel автоматически внедрит IssService
    public function __construct(IssService $issService)
    {
        $this->issService = $issService;
    }

    public function index()
    {
        // 1. Вызов Service Layer для получения агрегированных данных.
        $data = $this->issService->getIssDataForPage();
        
        // 2. Если в данных есть ошибка, можно перенаправить или показать заглушку.
        // В данном случае, Service уже вернул Fallback Data (пустые или из кэша).
        
        // 3. Возврат представления с готовыми ViewModel/DTO.
        return view('iss', $data);
    }
}