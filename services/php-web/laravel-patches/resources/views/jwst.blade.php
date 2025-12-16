@extends('layouts.app')

@section('content')
<div class="container pb-5">
  <h3 class="mb-3">JWST — последние изображения</h3>

  <form id="jwstFilter" class="row g-2 align-items-center mb-3">
    <div class="col-auto">
      <select class="form-select form-select-sm" name="source" id="srcSel">
        <option value="jpg" selected>Все JPG</option>
        <option value="suffix">По суффиксу</option>
        <option value="program">По программе</option>
      </select>
    </div>
    <div class="col-auto">
      <input type="text" class="form-control form-control-sm" name="suffix" id="suffixInp" placeholder="_cal / _thumb" style="width:140px;display:none">
      <input type="text" class="form-control form-control-sm" name="program" id="progInp" placeholder="2734" style="width:110px;display:none">
    </div>
    <div class="col-auto">
      <select class="form-select form-select-sm" name="instrument" style="width:130px">
        <option value="">Любой инструмент</option>
        <option>NIRCam</option><option>MIRI</option><option>NIRISS</option><option>NIRSpec</option><option>FGS</option>
      </select>
    </div>
    <div class="col-auto">
      <select class="form-select form-select-sm" name="perPage" style="width:90px">
        <option>12</option><option selected>24</option><option>36</option><option>48</option>
      </select>
    </div>
    <div class="col-auto">
      <button class="btn btn-sm btn-primary" type="submit">Показать</button>
    </div>
  </form>

  <div class="jwst-slider position-relative">
    <button class="btn btn-light border jwst-nav jwst-prev" type="button" aria-label="Prev">‹</button>
    <div id="jwstTrack" class="jwst-track border rounded"></div>
    <button class="btn btn-light border jwst-nav jwst-next" type="button" aria-label="Next">›</button>
  </div>
  <div id="jwstInfo" class="small text-muted mt-2"></div>
</div>

<script>
document.addEventListener('DOMContentLoaded', async function () {
  const track = document.getElementById('jwstTrack');
  const info  = document.getElementById('jwstInfo');
  const form  = document.getElementById('jwstFilter');
  const srcSel = document.getElementById('srcSel');
  const sfxInp = document.getElementById('suffixInp');
  const progInp= document.getElementById('progInp');

  function toggleInputs(){
    sfxInp.style.display  = (srcSel.value==='suffix')  ? '' : 'none';
    progInp.style.display = (srcSel.value==='program') ? '' : 'none';
  }
  srcSel.addEventListener('change', toggleInputs); toggleInputs();

  async function loadFeed(qs){
    track.innerHTML = '<div class="p-3 text-muted">Загрузка…</div>';
    info.textContent= '';
    try{
      const url = '/api/jwst/feed?'+new URLSearchParams(qs).toString();
      const r = await fetch(url);
      const js = await r.json();
      track.innerHTML = '';
      (js.items||[]).forEach(it=>{
        const fig = document.createElement('figure');
        fig.className = 'jwst-item m-0';
        fig.innerHTML = `
          <a href="${it.link||it.url}" target="_blank" rel="noreferrer">
            <img loading="lazy" src="${it.url}" alt="JWST">
          </a>
          <figcaption class="jwst-cap">${(it.caption||'').replaceAll('<','&lt;')}</figcaption>`;
        track.appendChild(fig);
      });
      info.textContent = `Источник: ${js.source} · Показано ${js.count||0}`;
    }catch(e){
      track.innerHTML = '<div class="p-3 text-danger">Ошибка загрузки</div>';
    }
  }

  form.addEventListener('submit', function(ev){
    ev.preventDefault();
    const fd = new FormData(form);
    loadFeed(Object.fromEntries(fd.entries()));
  });

  document.querySelector('.jwst-prev').addEventListener('click', ()=> track.scrollBy({left:-600, behavior:'smooth'}));
  document.querySelector('.jwst-next').addEventListener('click', ()=> track.scrollBy({left: 600, behavior:'smooth'}));

  loadFeed({source:'jpg', perPage:24});
});
</script>

<style>
.jwst-slider{position:relative}
.jwst-track{display:flex; gap:.75rem; overflow:auto; scroll-snap-type:x mandatory; padding:.25rem;}
.jwst-item{flex:0 0 180px; scroll-snap-align:start}
.jwst-item img{width:100%; height:180px; object-fit:cover; border-radius:.5rem}
.jwst-cap{font-size:.85rem; margin-top:.25rem}
.jwst-nav{position:absolute; top:40%; transform:translateY(-50%); z-index:2}
.jwst-prev{left:-.25rem} .jwst-next{right:-.25rem}
</style>
@endsection
