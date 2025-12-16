@extends('layouts.app')

@section('content')
<div class="container py-4">
  <h3 class="mb-3">Астрономические события</h3>

  <div class="row g-2 mb-3">
    <div class="col-md-2">
      <input id="astroLat" type="number" step="0.0001" class="form-control" placeholder="Широта" value="55.7558">
    </div>
    <div class="col-md-2">
      <input id="astroLon" type="number" step="0.0001" class="form-control" placeholder="Долгота" value="37.6176">
    </div>
    <div class="col-md-2">
      <input id="astroDays" type="number" class="form-control" placeholder="Дней" value="7">
    </div>
    <div class="col-md-2">
      <input id="astroLimit" type="number" class="form-control" placeholder="Лимит" value="50">
    </div>
    <div class="col-md-2">
      <button id="astroLoad" class="btn btn-primary w-100">Загрузить</button>
    </div>
    <div class="col-md-2">
      <input id="astroSearch" class="form-control" placeholder="Поиск">
    </div>
  </div>

  <select id="astroSort" class="form-select mb-3" style="max-width:200px">
    <option value="asc">По имени ↑</option>
    <option value="desc">По имени ↓</option>
  </select>

  <table class="table table-sm table-striped">
    <thead>
      <tr>
        <th>Название</th>
        <th>Тип</th>
        <th>Масса</th>
      </tr>
    </thead>
    <tbody id="astroBody"></tbody>
  </table>
</div>

<script>
let rows = [];

function render(data) {
  const body = document.getElementById('astroBody');
  body.innerHTML = '';
  data.forEach(r => {
    body.insertAdjacentHTML('beforeend', `
      <tr>
        <td>${r.englishName ?? '—'}</td>
        <td>${r.bodyType ?? '—'}</td>
        <td>${r.mass?.massValue ?? '—'}</td>
      </tr>
    `);
  });
}

function loadEvents() {
  const lat = document.getElementById('astroLat').value;
  const lon = document.getElementById('astroLon').value;
  const days = document.getElementById('astroDays').value;
  const limit = document.getElementById('astroLimit').value;

  fetch(`/api/astro/events?lat=${lat}&lon=${lon}&days=${days}&limit=${limit}`)
    .then(r => r.json())
    .then(data => {
      if(data.ok === false) {
        alert('Ошибка: ' + data.error);
        return;
      }
      rows = data;
      render(rows);
    });
}

document.getElementById('astroLoad').addEventListener('click', loadEvents);

document.getElementById('astroSearch').addEventListener('input', e => {
  const q = e.target.value.toLowerCase();
  render(rows.filter(r =>
    (r.englishName ?? '').toLowerCase().includes(q)
  ));
});

document.getElementById('astroSort').addEventListener('change', e => {
  const sorted = [...rows].sort((a,b)=>{
    const A = a.englishName ?? '';
    const B = b.englishName ?? '';
    return e.target.value === 'asc'
      ? A.localeCompare(B)
      : B.localeCompare(A);
  });
  render(sorted);
});

// Загрузка при старте
loadEvents();
</script>
@endsection
