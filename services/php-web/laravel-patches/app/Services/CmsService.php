<?php

namespace App\Services;

use App\Repositories\CmsRepository;
use Exception;

/**
 * Service Layer для работы с CMS.
 * Содержит бизнес-логику (если она появится), кэширование, и вызывает Репозиторий.
 */
class CmsService
{
    private CmsRepository $cmsRepository;

    public function __construct(CmsRepository $cmsRepository)
    {
        // Dependency Injection: получаем CMS Repository
        $this->cmsRepository = $cmsRepository;
    }

    /**
     * Получает данные страницы по slug и подготавливает их для View.
     *
     * @param string $slug
     * @return array
     * @throws \Illuminate\Http\Exceptions\HttpResponseException (или другое исключение 404)
     */
    public function getPageData(string $slug): array
    {
        $page = $this->cmsRepository->getActivePageBySlug($slug);

        if (!$page) {
            // В сервисе лучше бросать исключение, а не вызывать abort() напрямую.
            // Контроллер или Middleware должны перехватить его и вызвать abort(404).
            throw new Exception('CMS page not found.', 404);
        }

        // Маппинг и подготовка данных (ViewModel)
        return [
            'title' => $page->title,
            'html' => $page->content, // Мы не выполняем дополнительную обработку (например, Markdown->HTML)
        ];
    }
}