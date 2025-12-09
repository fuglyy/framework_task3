<?php

namespace App\Exceptions;

use Illuminate\Foundation\Exceptions\Handler as ExceptionHandler;
use Throwable;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class Handler extends ExceptionHandler
{
    /**
     * Список типов исключений, которые не должны быть переданы в отчет.
     * @var array<int, class-string<Throwable>>
     */
    protected $dontReport = [
        //
    ];

    /**
     * Список типов исключений, которые не будут преобразованы в HTTP-ответ.
     * @var array<int, class-string<Throwable>>
     */
    protected $dontFlash = [
        'current_password',
        'password',
        'password_confirmation',
    ];

    /**
     * Регистрирует функции обратного вызова для отчетов об исключениях.
     */
    public function register(): void
    {
        $this->reportable(function (Throwable $e) {
            //
        });
    }

    /**
     * Преобразует исключение в HTTP-ответ.
     * ЭТОТ МЕТОД ИМЕЕТ КРИТИЧЕСКОЕ ЗНАЧЕНИЕ ДЛЯ ВАШЕГО ТРЕБОВАНИЯ.
     *
     * @param  Request  $request
     * @param  Throwable $e
     * @return \Symfony\Component\HttpFoundation\Response
     */
    public function render($request, Throwable $e)
    {
        // Применяем логику только для API-запросов (например, начинающихся с /api)
        // В противном случае используем стандартный обработчик.
        if ($request->is('api/*')) {
            return $this->handleApiException($request, $e);
        }

        return parent::render($request, $e);
    }

    /**
     * Обрабатывает API-исключения, форматируя их как {"ok": false, ...} с HTTP 200.
     * @param Request $request
     * @param Throwable $e
     * @return JsonResponse
     */
    private function handleApiException($request, Throwable $e): JsonResponse
    {
        // 1. Извлекаем код ошибки и сообщение
        $statusCode = $e->getCode() ?: 500;
        $errorMessage = $e->getMessage() ?: 'Произошла непредвиденная ошибка сервера.';

        // Если это исключение аутентификации или авторизации, используем стандартные коды
        if ($this->isHttpException($e)) {
            $statusCode = $e->getStatusCode();
        } elseif ($statusCode < 400 || $statusCode >= 600) {
            // Если код не является HTTP-статусом или является невалидным, устанавливаем 500
            $statusCode = 500;
        }

        // 2. Логируем критические ошибки (5xx)
        if ($statusCode >= 500) {
            \Log::error("API Error [{$statusCode}]: {$errorMessage}", ['exception' => $e]);
        }
        
        // 3. Форматируем ответ согласно требованию: HTTP 200, но с ok: false в теле
        $responsePayload = [
            'ok' => false,
            'error' => $errorMessage,
            // Передаем статус-код в теле, чтобы клиент мог его прочитать
            'status_code' => $statusCode, 
            'path' => $request->path(),
        ];
        
        // В режиме отладки можем добавить трассировку стека
        if (config('app.debug')) {
             $responsePayload['trace'] = $e->getTrace();
        }

        // Возвращаем JSON с HTTP-статусом 200, но с ошибкой в теле
        return response()->json($responsePayload, 200);
    }
}