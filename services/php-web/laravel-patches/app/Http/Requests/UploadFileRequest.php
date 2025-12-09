<?php

namespace App\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

/**
 * Form Request для строгой валидации загружаемого файла.
 * Устраняет проблему "Intentionally weak validation".
 */
class UploadFileRequest extends FormRequest
{
    /**
     * Определяет, авторизован ли пользователь для выполнения этого запроса.
     * @return bool
     */
    public function authorize(): bool
    {
        // Замените на реальную логику авторизации
        return true; 
    }

    /**
     * Правила валидации, применимые к запросу.
     * @return array
     */
    public function rules(): array
    {
        return [
            // Строго требуем файл
            'file' => [
                'required', 
                'file', 
                'max:10240', // Максимальный размер 10MB
                'mimes:jpg,jpeg,png,gif,pdf,zip', // Разрешенные типы файлов (строго ограничены)
            ],
        ];
    }
    
    /**
     * Сообщения об ошибках.
     */
    public function messages(): array
    {
        return [
            'file.required' => 'Файл для загрузки не найден.',
            'file.max' => 'Размер файла не должен превышать 10 МБ.',
            'file.mimes' => 'Разрешены только файлы типов: jpg, png, gif, pdf, zip.',
        ];
    }
}