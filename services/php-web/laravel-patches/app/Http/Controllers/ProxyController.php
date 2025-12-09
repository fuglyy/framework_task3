<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use App\Services\ProxyService;
use Illuminate\Http\JsonResponse;

/**
 * ProxyController: "Тонкий" контроллер.
 * Его задача - принять запрос, вызвать соответствующий Сервис и вернуть JSON-ответ.
 * Вся логика проксирования и HTTP-вызовов перенесена в ProxyService/RustProxyClient.
 */
class ProxyController extends Controller
{
    private ProxyService $proxyService;

    public function __construct(ProxyService $proxyService)
    {
        // Dependency Injection
        $this->proxyService = $proxyService;
    }

    /**
     * Проксирует запрос /last.
     */
    public function last(): JsonResponse
    {
        // 1. Вызываем Service Layer
        $result = $this->proxyService->getLastData();

        // 2. Возвращаем ответ. Если Service бросит исключение, его поймает Handler,
        // который вернет {"ok": false, ...} с HTTP 200.
        return response()->json($result);
    }

    /**
     * Проксирует запрос /iss/trend с параметрами.
     * @param Request $request
     */
    public function trend(Request $request): JsonResponse
    {
        // 1. Получаем все параметры запроса
        $qs = $request->query();

        // 2. Вызываем Service Layer
        $result = $this->proxyService->getTrendData($qs);

        // 3. Возвращаем ответ.
        return response()->json($result);
    }
}