<?php

namespace App\Http\Controllers;

use App\Services\UploadService;
use App\Http\Requests\UploadFileRequest; // Используем Form Request

/**
 * UploadController: "Тонкий" контроллер.
 * Отвечает только за прием запроса и передачу файла в Service Layer.
 */
class UploadController extends Controller
{
    private UploadService $uploadService;

    public function __construct(UploadService $uploadService)
    {
        // Dependency Injection
        $this->uploadService = $uploadService;
    }

    /**
     * Обрабатывает загрузку файла.
     *
     * @param UploadFileRequest $request Request, который уже прошел строгую валидацию.
     * @return \Illuminate\Http\RedirectResponse
     */
    public function store(UploadFileRequest $request)
    {
        // 1. Валидация уже прошла автоматически благодаря UploadFileRequest
        $file = $request->file('file');
        
        try {
            // 2. Вызов Service Layer для сохранения файла.
            $path = $this->uploadService->storeFile($file);

            // 3. Успешный ответ
            return back()->with('status', 'Файл успешно загружен по пути: ' . $path);
            
        } catch (\Exception $e) {
            // Обработка ошибок файловой системы
            // В реальном приложении здесь нужна более детальная обработка
            return back()->with('status', 'Ошибка загрузки файла: ' . $e->getMessage());
        }
    }
}