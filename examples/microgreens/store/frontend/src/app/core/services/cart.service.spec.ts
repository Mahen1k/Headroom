import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { provideHttpClient } from '@angular/common/http';
import { TestBed } from '@angular/core/testing';
import { environment } from '../../../environments/environment';
import { CartService } from './cart.service';

describe('CartService', () => {
  let service: CartService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [CartService, provideHttpClient(), provideHttpClientTesting()],
    });
    service = TestBed.inject(CartService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => httpMock.verify());

  it('fetches the cart', () => {
    service.getCart().subscribe((items) => {
      expect(items.length).toBe(1);
      expect(items[0].variety).toBe('sunflower');
    });

    const req = httpMock.expectOne(`${environment.apiUrl}/cart`);
    expect(req.request.method).toBe('GET');
    req.flush([
      { id: 1, batch_id: 1, variety: 'sunflower', quantity: 2, unit_price: 4.5, line_total: 9 },
    ]);
  });

  it('adds an item to the cart with the requested quantity', () => {
    service.addToCart(1, 3).subscribe();

    const req = httpMock.expectOne(`${environment.apiUrl}/cart/items`);
    expect(req.request.method).toBe('POST');
    expect(req.request.body).toEqual({ batch_id: 1, quantity: 3 });
    req.flush([]);
  });

  it('removes a cart item by id', () => {
    service.removeFromCart(5).subscribe();

    const req = httpMock.expectOne(`${environment.apiUrl}/cart/items/5`);
    expect(req.request.method).toBe('DELETE');
    req.flush({ removed: true });
  });

  it('checks out with a discount code', () => {
    service.checkout('FRESH10').subscribe((order) => {
      expect(order.total).toBe(12.15);
    });

    const req = httpMock.expectOne(`${environment.apiUrl}/checkout`);
    expect(req.request.method).toBe('POST');
    expect(req.request.body).toEqual({ discount_code: 'FRESH10' });
    req.flush({ id: 1, subtotal: 13.5, total: 12.15, discount_code: 'FRESH10', items: [] });
  });

  it('checks out without a discount code', () => {
    service.checkout(null).subscribe();

    const req = httpMock.expectOne(`${environment.apiUrl}/checkout`);
    expect(req.request.body).toEqual({ discount_code: null });
    req.flush({ id: 2, subtotal: 4.5, total: 4.5, discount_code: null, items: [] });
  });

  it('lists past orders', () => {
    service.listOrders().subscribe((orders) => {
      expect(orders.length).toBe(1);
    });

    const req = httpMock.expectOne(`${environment.apiUrl}/orders`);
    expect(req.request.method).toBe('GET');
    req.flush([{ id: 1, subtotal: 4.5, total: 4.5, discount_code: null, items: [] }]);
  });
});
