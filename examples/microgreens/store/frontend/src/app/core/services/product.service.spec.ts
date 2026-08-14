import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { provideHttpClient } from '@angular/common/http';
import { TestBed } from '@angular/core/testing';
import { environment } from '../../../environments/environment';
import { ProductService } from './product.service';

describe('ProductService', () => {
  let service: ProductService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [ProductService, provideHttpClient(), provideHttpClientTesting()],
    });
    service = TestBed.inject(ProductService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => httpMock.verify());

  it('lists salable products', () => {
    service.list().subscribe((products) => {
      expect(products.length).toBe(1);
      expect(products[0].grade).toBe('A');
    });

    const req = httpMock.expectOne(`${environment.apiUrl}/products`);
    expect(req.request.method).toBe('GET');
    req.flush([
      {
        id: 1,
        variety: 'sunflower',
        sow_date: '2026-08-04',
        days_since_sowing: 10,
        quantity_trays: 7,
        price_per_tray: 4.5,
        grade: 'A',
      },
    ]);
  });

  it('lists known varieties', () => {
    service.varieties().subscribe((varieties) => {
      expect(varieties).toContain('sunflower');
    });

    const req = httpMock.expectOne(`${environment.apiUrl}/varieties`);
    expect(req.request.method).toBe('GET');
    req.flush(['sunflower', 'pea', 'radish']);
  });
});
