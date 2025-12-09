<?php

namespace App\Services;

use Illuminate\Http\UploadedFile;
use Illuminate\Support\Facades\Storage;
use Illuminate\Support\Str;

/**
 * Service Layer для работы с файловой системой и загрузкой.
 * Инкапсулирует логику сохранения файлов.
 */
class UploadService
{
    private const UPLOAD_DISK = 'public'; // Используем Laravel Filesystem для гибкости
    private const UPLOAD_FOLDER = 'uploads';

    /**
     * Сохраняет загруженный файл и возвращает его путь.
     *
     * @param UploadedFile $file Файл, прошедший валидацию.
     * @return string Полный путь к сохраненному файлу относительно корневого каталога.
     */
    public function storeFile(UploadedFile $file): string
    {
        // 1. Создаем безопасное имя файла, чтобы избежать проблем.
        // Используем оригинальное имя + timestamp, чтобы минимизировать коллизии и сохранить читаемость.
        $originalName = $file->getClientOriginalName();
        $safeName = time() . '_' . Str::slug(pathinfo($originalName, PATHINFO_FILENAME)) . '.' . $file->getClientOriginalExtension();

        // 2. Использование Laravel Storage для абстракции от файловой системы
        // (теперь легко перейти на S3 или Azure Storage)
        $path = $file->storeAs(self::UPLOAD_FOLDER, $safeName, self::UPLOAD_DISK);

        return $path;
    }
}