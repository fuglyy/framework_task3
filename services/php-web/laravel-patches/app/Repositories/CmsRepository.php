<?php

namespace App\Repositories;

use Illuminate\Support\Facades\DB;
use stdClass;

/**
 * Репозиторий для доступа к данным CMS (cms_blocks).
 * Инкапсулирует все SQL-запросы.
 */
class CmsRepository
{
    /**
     * Получает одну активную страницу CMS по её slug.
     *
     * @param string $slug Уникальный идентификатор страницы.
     * @return stdClass|null Объект с полями 'title' и 'content', или null, если не найдено.
     */
    public function getActivePageBySlug(string $slug): ?stdClass
    {
        // Использование параметризованного запроса для защиты от SQL-инъекций (как и было в оригинале)
        $row = DB::selectOne("
            SELECT title, content 
            FROM cms_blocks 
            WHERE slug = ? 
            AND is_active = TRUE
        ", [$slug]);

        return $row ?: null;
    }

    // Здесь могут быть добавлены другие методы: updatePage, getAllPages, deletePage, и тp.
}