<?php

namespace App\Http\Controllers;

use App\Services\CmsService;
use Exception;

/**
 * CmsController: "Тонкий" контроллер.
 * Логика доступа к базе данных перенесена в CmsRepository.
 */
class CmsController extends Controller
{
    private CmsService $cmsService;

    // Dependency Injection: Laravel автоматически внедрит CmsService
    public function __construct(CmsService $cmsService)
    {
        $this->cmsService = $cmsService;
    }

    public function page(string $slug)
    {
        try {
            // 1. Вызов Service Layer.
            $data = $this->cmsService->getPageData($slug);
            
            // 2. Успешный возврат представления с готовыми данными.
            return response()->view('cms.page', $data);
            
        } catch (Exception $e) {
            // 3. Обработка ошибки "страница не найдена", проброшенной из Service.
            if ($e->getCode() === 404) {
                // Вызываем Laravel's abort, как и было в оригинале, но теперь логично.
                abort(404);
            }
            // Для других ошибок (например, DB connection failure)
            throw $e;
        }
    }
}