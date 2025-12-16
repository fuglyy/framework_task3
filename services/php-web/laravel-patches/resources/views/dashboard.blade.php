@extends('layouts.app')

@section('content')
<div class="container py-4 fade-in">
  <h3 class="mb-4">Дашборды</h3>

  <div class="row g-3">
    <div class="col-md-4">
      <a href="/iss" class="card card-hover text-decoration-none">
        <div class="card-body">
          <h5>МКС</h5>
          <div class="text-muted">Положение и движение</div>
        </div>
      </a>
    </div>

    <div class="col-md-4">
      <a href="/jwst" class="card card-hover text-decoration-none">
        <div class="card-body">
          <h5>JWST</h5>
          <div class="text-muted">Изображения телескопа</div>
        </div>
      </a>
    </div>

    <div class="col-md-4">
      <a href="/astro" class="card card-hover text-decoration-none">
        <div class="card-body">
          <h5>Астрособытия</h5>
          <div class="text-muted">AstronomyAPI</div>
        </div>
      </a>
    </div>
  </div>
</div>
@endsection
